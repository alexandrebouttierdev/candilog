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
        Some(raw) => serde_json::from_str::<SettingsStockes>(&raw)
            .map(|p| p.llm)
            .map_err(|_| {
                AppError::Provider("Les réglages IA enregistrés sont illisibles".into())
            })?,
        None => LlmConfig::default(),
    };
    config.normaliser_legacy();
    // Ollama n'interroge pas le trousseau : CI et tests n'ont souvent aucun service de secrets.
    if !matches!(
        config.provider,
        ProviderKind::Ollama | ProviderKind::MistralLocal
    ) && config
        .api_key
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        config.api_key = secret_store.load_api_key()?;
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

    fn pool() -> SqlitePool {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        pool
    }

    #[test]
    fn une_base_neuve_retombe_sur_ollama_local() {
        let config = load_config(&pool()).unwrap();
        assert_eq!(
            config.provider,
            crate::features::ai::domain::ProviderKind::Ollama
        );
        assert_eq!(config.model, "");
    }

    #[test]
    fn un_json_vide_n_est_pas_une_erreur() {
        let pool = pool();
        connection(&pool)
            .unwrap()
            .execute(
                "INSERT INTO settings (id, data, updated_at) VALUES (1, '{}', datetime('now'))",
                [],
            )
            .unwrap();
        assert_eq!(
            load_config(&pool).unwrap().provider,
            crate::features::ai::domain::ProviderKind::Ollama
        );
    }

    /// Chaîne complète du bogue signalé : la grille des réglages vide `llm.model` en
    /// sélectionnant Mistral Local, l'enregistrement est accepté, mais toute opération IA
    /// repassait ensuite par `est_configure` et échouait sur « Configurez un fournisseur ».
    #[test]
    fn mistral_local_enregistre_sans_modele_reste_utilisable() {
        let pool = pool();
        connection(&pool)
            .unwrap()
            .execute(
                "INSERT INTO settings (id, data, updated_at) VALUES (1, ?1, datetime('now'))",
                [r#"{"llm":{"provider":"mistral_local","api_key":null,"endpoint":null,"model":"","temperature":0.7}}"#],
            )
            .unwrap();
        let config = load_config(&pool).expect("Mistral Local doit rester utilisable");
        assert_eq!(config.provider, ProviderKind::MistralLocal);
    }

    struct CoffreFixe(Option<String>);

    impl SecretStoreContract for CoffreFixe {
        fn load_api_key(&self) -> AppResult<Option<String>> {
            Ok(self.0.clone())
        }
        fn store_api_key(&self, _: Option<&str>) -> AppResult<()> {
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
