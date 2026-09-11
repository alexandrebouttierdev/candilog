//! Lecture non destructive de la configuration LLM stockée dans `parametres`.

use crate::core::database::helpers::connection;
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::core::secrets::{SecretStore, SecretStoreContract};
use crate::features::ai::domain::{LlmConfig, ProviderKind, SettingsStockes};
use rusqlite::OptionalExtension;

pub fn load_config(pool: &SqlitePool) -> AppResult<LlmConfig> {
    load_config_avec(pool, &SecretStore)
}

pub fn load_config_avec(
    pool: &SqlitePool,
    secret_store: &impl SecretStoreContract,
) -> AppResult<LlmConfig> {
    let raw: Option<String> = connection(pool)?
        .query_row("SELECT data FROM settings WHERE id = 1", [], |row| {
            row.get(0)
        })
        .optional()?;
    let mut config = match raw {
        Some(raw) => {
            let prepared = crate::features::settings::domain::preparer_settings_json(&raw);
            serde_json::from_str::<SettingsStockes>(&prepared)
        }
        .map(|p| p.llm)
        .map_err(|_| AppError::Provider("Les réglages IA enregistrés sont illisibles".into()))?,
        None => LlmConfig::default(),
    };
    config.normaliser_legacy();
    // Ollama n'interroge pas le trousseau : CI et tests n'ont souvent aucun service de secrets.
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
    if !config.est_configure() {
        return Err(AppError::Provider(
            "Configurez un fournisseur IA dans Réglages avant de lancer cette opération".into(),
        ));
    }
    Ok(config)
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
}
