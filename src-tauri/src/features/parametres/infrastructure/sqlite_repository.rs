//! Table singleton `parametres`, JSON historique conservé tel quel.

use crate::core::database::helpers::{connexion, maintenant_iso, traduire_erreur};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::parametres::domain::{AppSettings, ParametresRepository};
use rusqlite::OptionalExtension;

pub struct SqliteParametresRepository {
    pool: SqlitePool,
}

impl SqliteParametresRepository {
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl ParametresRepository for SqliteParametresRepository {
    fn get(&self) -> AppResult<AppSettings> {
        let conn = connexion(&self.pool)?;
        let contenu: Option<String> = conn
            .query_row("SELECT data FROM parametres WHERE id = 1", [], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|e| traduire_erreur(e, "paramètres"))?;
        let Some(texte) = contenu else {
            return Ok(AppSettings::default());
        };
        match serde_json::from_str(&texte) {
            Ok(settings) => Ok(settings),
            Err(error) => {
                tracing::warn!(%error, "paramètres illisibles, valeurs par défaut");
                conn.execute(
                    "INSERT INTO app_kv (cle, valeur) VALUES ('parametres_corrompus', ?1)
                     ON CONFLICT(cle) DO UPDATE SET valeur = excluded.valeur",
                    [&texte],
                )
                .map_err(|e| traduire_erreur(e, "sauvegarde des paramètres illisibles"))?;
                Ok(AppSettings::default())
            }
        }
    }

    fn upsert(&self, settings: &AppSettings) -> AppResult<AppSettings> {
        let conn = connexion(&self.pool)?;
        let maintenant = maintenant_iso();
        let contenu =
            serde_json::to_string(settings).map_err(|e| AppError::Serialization(e.to_string()))?;
        conn.execute(
            "INSERT INTO parametres (id, data, updated_at) VALUES (1, ?1, ?2)
             ON CONFLICT(id) DO UPDATE SET data = excluded.data, updated_at = excluded.updated_at",
            rusqlite::params![contenu, maintenant],
        )
        .map_err(|e| traduire_erreur(e, "paramètres"))?;
        Ok(settings.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::{open_pool, run_local_migrations};
    use crate::features::ia::domain::ProviderKind;

    fn repo() -> SqliteParametresRepository {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        SqliteParametresRepository::new(pool)
    }

    #[test]
    fn sans_ligne_retourne_ollama_par_defaut() {
        let settings = repo().get().unwrap();
        assert_eq!(settings.llm.provider, ProviderKind::Ollama);
        assert_eq!(settings.langue, "fr");
    }

    #[test]
    fn upsert_remplace_la_ligne_unique() {
        let repo = repo();
        repo.upsert(&AppSettings {
            langue: "en".into(),
            ..AppSettings::default()
        })
        .unwrap();
        repo.upsert(&AppSettings {
            langue: "fr".into(),
            ..AppSettings::default()
        })
        .unwrap();
        assert_eq!(repo.get().unwrap().langue, "fr");
    }

    #[test]
    fn json_illisible_retombe_sur_les_defauts() {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        {
            let conn = connexion(&pool).unwrap();
            conn.execute(
                "INSERT INTO parametres (id, data, updated_at) VALUES (1, 'pas du json', datetime('now'))",
                [],
            )
            .unwrap();
        }
        let settings = SqliteParametresRepository::new(pool).get().unwrap();
        assert_eq!(settings.llm.provider, ProviderKind::Ollama);
    }
}
