//! Lecture non destructive de la configuration LLM stockée dans `parametres`.

use crate::core::database::helpers::connection;
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::core::secrets::{SecretStore, SecretStoreContract};
use crate::features::ai::domain::{route_config, AiTask, LlmConfig, ProviderKind, SettingsStockes};
use rusqlite::OptionalExtension;

pub fn load_config(pool: &SqlitePool) -> AppResult<LlmConfig> {
    load_config_avec(pool, &SecretStore)
}

pub fn load_config_avec(
    pool: &SqlitePool,
    secret_store: &impl SecretStoreContract,
) -> AppResult<LlmConfig> {
    let mut config = read_settings(pool)?.llm;
    config.normaliser_legacy();
    inject_api_key(&mut config, secret_store)?;
    if !config.est_configure() {
        return Err(AppError::Provider(
            "Configurez un fournisseur IA dans Réglages avant de lancer cette opération".into(),
        ));
    }
    Ok(config)
}

/// Configuration d'une tâche, et `true` si elle vient d'une route explicite.
///
/// Sans route, la tâche suit le fournisseur principal. Une tâche désactivée ou routée vers
/// un fournisseur incomplet **s'arrête et le dit** : aucun repli vers un autre fournisseur,
/// qui enverrait des données là où l'utilisateur ne l'a pas décidé.
pub fn load_task_config(pool: &SqlitePool, task: AiTask) -> AppResult<(LlmConfig, bool)> {
    load_task_config_avec(pool, task, &SecretStore)
}

pub fn load_task_config_avec(
    pool: &SqlitePool,
    task: AiTask,
    secret_store: &impl SecretStoreContract,
) -> AppResult<(LlmConfig, bool)> {
    let stored = read_settings(pool)?;
    let route = match stored.ai_routes.get(&task) {
        None => return load_config_avec(pool, secret_store).map(|config| (config, false)),
        Some(None) => {
            return Err(AppError::Provider(format!(
            "La tâche « {} » est désactivée. Choisissez un modèle dans Intelligence artificielle.",
            task.label()
        )))
        }
        Some(Some(route)) => route,
    };
    let mut main = stored.llm;
    main.normaliser_legacy();
    let mut config = route_config(route, &main, &stored.llm_presets);
    inject_api_key(&mut config, secret_store)?;
    if !config.est_configure() {
        return Err(AppError::Provider(format!(
            "« {} » attend sa configuration : {} n'a pas de clé API ou de modèle. Complétez-le dans Intelligence artificielle.",
            task.label(),
            provider_label(&config.provider)
        )));
    }
    Ok((config, true))
}

/// Nom du fournisseur dans les messages.
fn provider_label(provider: &ProviderKind) -> String {
    match provider {
        ProviderKind::CandilogLocal => "l'IA locale".into(),
        ProviderKind::Ollama => "Ollama".into(),
        ProviderKind::Claude => "Anthropic".into(),
        ProviderKind::OpenAI => "OpenAI".into(),
        ProviderKind::Gemini => "Gemini".into(),
        ProviderKind::Mistral => "Mistral AI".into(),
        ProviderKind::DeepSeek => "DeepSeek".into(),
        ProviderKind::Custom(name) => name.clone(),
    }
}

fn read_settings(pool: &SqlitePool) -> AppResult<SettingsStockes> {
    let raw: Option<String> = connection(pool)?
        .query_row("SELECT data FROM settings WHERE id = 1", [], |row| {
            row.get(0)
        })
        .optional()?;
    match raw {
        Some(raw) => {
            let prepared = crate::features::settings::domain::preparer_settings_json(&raw);
            serde_json::from_str::<SettingsStockes>(&prepared).map_err(|_| {
                AppError::Provider("Les réglages IA enregistrés sont illisibles".into())
            })
        }
        None => Ok(SettingsStockes {
            llm: LlmConfig::default(),
            llm_presets: std::collections::BTreeMap::new(),
            ai_routes: std::collections::BTreeMap::new(),
        }),
    }
}

/// Charge la clé d'un fournisseur distant depuis le coffre. Ollama et l'IA locale n'en ont
/// pas : CI et tests n'ont souvent aucun service de secrets.
fn inject_api_key(
    config: &mut LlmConfig,
    secret_store: &impl SecretStoreContract,
) -> AppResult<()> {
    if !matches!(
        config.provider,
        ProviderKind::Ollama | ProviderKind::CandilogLocal
    ) && config
        .api_key
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        config.api_key = secret_store.load_api_key(config.provider.storage_id())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::{open_pool, run_local_migrations};
    use crate::features::settings::domain::{AppSettings, SettingsRepository};
    use crate::features::settings::infrastructure::SqliteSettingsRepository;

    fn pool() -> SqlitePool {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        pool
    }

    #[test]
    fn une_base_neuve_utilise_candilog_local_par_defaut() {
        let config = load_config(&pool()).unwrap();
        assert_eq!(config.provider, ProviderKind::CandilogLocal);
        assert!(config.model.is_empty());
    }

    #[test]
    fn un_json_vide_utilise_candilog_local_par_defaut() {
        let pool = pool();
        connection(&pool)
            .unwrap()
            .execute(
                "INSERT INTO settings (id, data, updated_at) VALUES (1, '{}', datetime('now'))",
                [],
            )
            .unwrap();
        let config = load_config(&pool).unwrap();
        assert_eq!(config.provider, ProviderKind::CandilogLocal);
    }

    #[test]
    fn ollama_sans_modele_reste_non_configure() {
        let pool = pool();
        connection(&pool)
            .unwrap()
            .execute(
                "INSERT INTO settings (id, data, updated_at) VALUES (1, ?1, datetime('now'))",
                [r#"{"llm":{"provider":"ollama","api_key":null,"endpoint":"http://localhost:11434","model":"","temperature":0.7}}"#],
            )
            .unwrap();
        assert!(matches!(load_config(&pool), Err(AppError::Provider(_))));
    }

    /// `AiService::provider` relit cette configuration avant chaque opération. Le second
    /// enregistrement doit donc remplacer le modèle, sans conserver une instance Ollama
    /// construite avec la valeur précédente.
    #[test]
    fn le_dernier_modele_ollama_enregistre_est_relu() {
        let pool = pool();
        let repository = SqliteSettingsRepository::new(pool.clone());
        let mut settings = AppSettings::default();
        settings.llm.provider = ProviderKind::Ollama;
        settings.llm.endpoint = Some("http://localhost:11434".into());
        settings.llm.model = "LiquidAI/lfm2.5-1.2b-instruct:latest".into();
        repository.upsert(&settings).unwrap();

        settings.llm.model = "maternion/lfm2.5:350m".into();
        repository.upsert(&settings).unwrap();

        let config = load_config(&pool).unwrap();
        assert_eq!(config.provider, ProviderKind::Ollama);
        assert_eq!(config.model, "maternion/lfm2.5:350m");
    }

    struct CoffreFixe(Option<String>);

    impl SecretStoreContract for CoffreFixe {
        fn load_api_key(&self, _provider_id: &str) -> AppResult<Option<String>> {
            Ok(self.0.clone())
        }
        fn store_api_key(&self, _: &str, _: Option<&str>) -> AppResult<()> {
            Ok(())
        }
        fn clear_all_api_keys(&self) -> AppResult<()> {
            Ok(())
        }
    }

    #[test]
    fn un_fournisseur_cloud_injecte_la_cle_du_coffre() {
        let pool = pool();
        connection(&pool)
            .unwrap()
            .execute(
                "INSERT INTO settings (id, data, updated_at) VALUES (1, ?1, datetime('now'))",
                [r#"{"llm":{"provider":"open_ai","api_key":null,"endpoint":null,"model":"gpt-4o","temperature":0.5}}"#],
            )
            .unwrap();
        let config = load_config_avec(&pool, &CoffreFixe(Some("sk-test".into()))).unwrap();
        assert_eq!(config.api_key.as_deref(), Some("sk-test"));
    }

    fn enregistrer(pool: &SqlitePool, json: &str) {
        connection(pool)
            .unwrap()
            .execute(
                "INSERT INTO settings (id, data, updated_at) VALUES (1, ?1, datetime('now'))",
                [json],
            )
            .unwrap();
    }

    #[test]
    fn une_tache_sans_route_suit_le_fournisseur_principal() {
        let pool = pool();
        enregistrer(
            &pool,
            r#"{"llm":{"provider":"ollama","api_key":null,"endpoint":"http://localhost:11434","model":"qwen","temperature":0.7}}"#,
        );
        let (config, routee) =
            load_task_config_avec(&pool, AiTask::AnalyzeResume, &CoffreFixe(None)).unwrap();
        assert!(!routee);
        assert_eq!(config.model, "qwen");
    }

    #[test]
    fn une_tache_routee_prend_le_fournisseur_et_le_modele_de_sa_route() {
        let pool = pool();
        enregistrer(
            &pool,
            r#"{"llm":{"provider":"candilog_local","api_key":null,"endpoint":null,"model":"","temperature":0.7},
                "llm_presets":{"claude":{"endpoint":null,"model":"claude-x","temperature":0.2,"mode":"standard"}},
                "ai_routes":{"analyze_resume":{"provider":"claude","model":"claude-sonnet"}}}"#,
        );
        let (config, routee) = load_task_config_avec(
            &pool,
            AiTask::AnalyzeResume,
            &CoffreFixe(Some("sk-ant".into())),
        )
        .unwrap();
        assert!(routee);
        assert_eq!(config.provider, ProviderKind::Claude);
        assert_eq!(config.model, "claude-sonnet");
        assert_eq!(config.api_key.as_deref(), Some("sk-ant"));
        assert!((config.temperature - 0.2).abs() < f32::EPSILON);
        // Les autres tâches restent sur le fournisseur principal.
        let (autre, _) =
            load_task_config_avec(&pool, AiTask::WriteLetter, &CoffreFixe(None)).unwrap();
        assert_eq!(autre.provider, ProviderKind::CandilogLocal);
    }

    #[test]
    fn une_route_sans_cle_s_arrete_au_lieu_de_basculer() {
        let pool = pool();
        enregistrer(
            &pool,
            r#"{"llm":{"provider":"candilog_local","api_key":null,"endpoint":null,"model":"","temperature":0.7},
                "ai_routes":{"analyze_resume":{"provider":"claude","model":"claude-sonnet"}}}"#,
        );
        let erreur = load_task_config_avec(&pool, AiTask::AnalyzeResume, &CoffreFixe(None))
            .unwrap_err()
            .to_string();
        assert!(erreur.contains("Analyser un CV"), "{erreur}");
        assert!(erreur.contains("Anthropic"), "{erreur}");
    }

    #[test]
    fn une_tache_desactivee_refuse_de_s_executer() {
        let pool = pool();
        enregistrer(
            &pool,
            r#"{"llm":{"provider":"candilog_local","api_key":null,"endpoint":null,"model":"","temperature":0.7},
                "ai_routes":{"extract_offer":null}}"#,
        );
        let erreur = load_task_config_avec(&pool, AiTask::ExtractOffer, &CoffreFixe(None))
            .unwrap_err()
            .to_string();
        assert!(erreur.contains("désactivée"), "{erreur}");
    }
}
