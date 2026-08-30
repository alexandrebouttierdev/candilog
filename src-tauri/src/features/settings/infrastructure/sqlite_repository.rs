//! Persistance des paramètres avec archivage expurgé en cas d'incompatibilité.

use crate::core::database::helpers::{connection, now_iso, translate_error};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::settings::domain::{AppSettings, SettingsRepository};
use rusqlite::OptionalExtension;

const CORRUPT_SETTINGS_REDACTION: &str =
    "[contenu illisible non archivé afin de protéger les secrets]";

fn is_sensitive_key(key: &str) -> bool {
    let normalized = key
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect::<String>();
    [
        "apikey",
        "token",
        "secret",
        "password",
        "credential",
        "authorization",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn redact_secret_fields(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(object) => {
            object.retain(|key, _| !is_sensitive_key(key));
            for child in object.values_mut() {
                redact_secret_fields(child);
            }
        }
        serde_json::Value::Array(array) => {
            for child in array {
                redact_secret_fields(child);
            }
        }
        _ => {}
    }
}

fn redact_corrupt_settings(text: &str) -> String {
    let Ok(mut value) = serde_json::from_str::<serde_json::Value>(text) else {
        return CORRUPT_SETTINGS_REDACTION.to_owned();
    };
    redact_secret_fields(&mut value);
    serde_json::to_string(&value).unwrap_or_else(|_| CORRUPT_SETTINGS_REDACTION.to_owned())
}

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
                let redacted_text = redact_corrupt_settings(&text);
                conn.execute(
                    "INSERT INTO app_kv (kv_key, kv_value) VALUES ('parametres_corrompus', ?1)
                     ON CONFLICT(kv_key) DO UPDATE SET kv_value = excluded.kv_value",
                    [&redacted_text],
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

    #[test]
    fn json_illisible_n_archive_jamais_un_secret() {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        {
            let conn = connection(&pool).unwrap();
            conn.execute(
                "INSERT INTO settings (id, data, updated_at) VALUES (1, ?1, datetime('now'))",
                [r#"{"llm":{"api_key":"sk-secret"},"broken":true"#],
            )
            .unwrap();
        }

        SqliteSettingsRepository::new(pool.clone()).get().unwrap();

        let archived: String = connection(&pool)
            .unwrap()
            .query_row(
                "SELECT kv_value FROM app_kv WHERE kv_key = 'parametres_corrompus'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!archived.contains("sk-secret"));
        assert!(!archived.contains("api_key"));
    }

    #[test]
    fn json_incompatible_archive_uniquement_les_champs_non_sensibles() {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        {
            let conn = connection(&pool).unwrap();
            conn.execute(
                "INSERT INTO settings (id, data, updated_at) VALUES (1, ?1, datetime('now'))",
                [r#"{"language":42,"llm":{"api_key":"sk-secret","model":"modele-local"}}"#],
            )
            .unwrap();
        }

        SqliteSettingsRepository::new(pool.clone()).get().unwrap();

        let archived: String = connection(&pool)
            .unwrap()
            .query_row(
                "SELECT kv_value FROM app_kv WHERE kv_key = 'parametres_corrompus'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(archived.contains("modele-local"));
        assert!(!archived.contains("sk-secret"));
        assert!(!archived.contains("api_key"));
    }
}
