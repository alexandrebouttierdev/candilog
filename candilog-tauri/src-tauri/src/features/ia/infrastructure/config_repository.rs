//! Lecture non destructive de la configuration LLM stockée dans `parametres`.

use crate::core::database::helpers::connexion;
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::ia::domain::{LlmConfig, ParametresStockes};
use rusqlite::OptionalExtension;

pub fn charger_config(pool: &SqlitePool) -> AppResult<LlmConfig> {
    let brut: Option<String> = connexion(pool)?
        .query_row("SELECT data FROM parametres WHERE id = 1", [], |row| {
            row.get(0)
        })
        .optional()?;
    let config = match brut {
        Some(brut) => serde_json::from_str::<ParametresStockes>(&brut)
            .map(|p| p.llm)
            .map_err(|_| {
                AppError::Provider("Les réglages IA enregistrés sont illisibles".into())
            })?,
        None => LlmConfig::default(),
    };
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
        let config = charger_config(&pool()).unwrap();
        assert_eq!(
            config.provider,
            crate::features::ia::domain::ProviderKind::Ollama
        );
        assert_eq!(config.model, "llama3.2:3b");
    }

    #[test]
    fn un_json_vide_n_est_pas_une_erreur() {
        let pool = pool();
        connexion(&pool)
            .unwrap()
            .execute(
                "INSERT INTO parametres (id, data, updated_at) VALUES (1, '{}', datetime('now'))",
                [],
            )
            .unwrap();
        assert_eq!(
            charger_config(&pool).unwrap().provider,
            crate::features::ia::domain::ProviderKind::Ollama
        );
    }
}
