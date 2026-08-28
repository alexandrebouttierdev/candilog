//! Table singleton `parametres`, JSON historique conservé tel quel.

use crate::core::database::helpers::{connection, now_iso, translate_error};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::settings::domain::{AppSettings, SettingsRepository};
use rusqlite::OptionalExtension;

pub struct SqliteSettingsRepository {
    pool: SqlitePool,
}

impl SqliteSettingsRepository {
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl SettingsRepository for SqliteSettingsRepository {
    fn get(&self) -> AppResult<AppSettings> {
        let conn = connection(&self.pool)?;
        let content: Option<String> = conn
            .query_row("SELECT data FROM settings WHERE id = 1", [], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|e| translate_error(e, "paramètres"))?;
        let Some(text) = content else {
            return Ok(AppSettings::default());
        };
        match serde_json::from_str(&text) {
            Ok(settings) => Ok(settings),
            Err(error) => {
                tracing::warn!(%error, "paramètres illisibles, valeurs par défaut");
                conn.execute(
                    "INSERT INTO app_kv (kv_key, kv_value) VALUES ('parametres_corrompus', ?1)
                     ON CONFLICT(kv_key) DO UPDATE SET kv_value = excluded.kv_value",
                    [&text],
                )
                .map_err(|e| translate_error(e, "sauvegarde des paramètres illisibles"))?;
                Ok(AppSettings::default())
            }
        }
    }

    fn upsert(&self, settings: &AppSettings) -> AppResult<AppSettings> {
        let conn = connection(&self.pool)?;
        let now = now_iso();
        let content =
            serde_json::to_string(settings).map_err(|e| AppError::Serialization(e.to_string()))?;
        conn.execute(
            "INSERT INTO settings (id, data, updated_at) VALUES (1, ?1, ?2)
             ON CONFLICT(id) DO UPDATE SET data = excluded.data, updated_at = excluded.updated_at",
            rusqlite::params![content, now],
        )
        .map_err(|e| translate_error(e, "paramètres"))?;
        Ok(settings.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::{open_pool, run_local_migrations};
    use crate::features::ai::domain::ProviderKind;

    fn repo() -> SqliteSettingsRepository {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        SqliteSettingsRepository::new(pool)
    }

    #[test]
    fn sans_ligne_retourne_ollama_par_defaut() {
        let settings = repo().get().unwrap();
        assert_eq!(settings.llm.provider, ProviderKind::Ollama);
        assert_eq!(settings.language, "fr");
    }

    #[test]
    fn upsert_remplace_la_ligne_unique() {
        let repo = repo();
        repo.upsert(&AppSettings {
            language: "en".into(),
            ..AppSettings::default()
        })
        .unwrap();
        repo.upsert(&AppSettings {
            language: "fr".into(),
            ..AppSettings::default()
        })
        .unwrap();
        assert_eq!(repo.get().unwrap().language, "fr");
    }

    #[test]
    fn json_illisible_retombe_sur_les_defauts() {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        {
            let conn = connection(&pool).unwrap();
            conn.execute(
                "INSERT INTO settings (id, data, updated_at) VALUES (1, 'pas du json', datetime('now'))",
                [],
            )
            .unwrap();
        }
        let settings = SqliteSettingsRepository::new(pool).get().unwrap();
        assert_eq!(settings.llm.provider, ProviderKind::Ollama);
    }
}
