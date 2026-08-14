//! Accès SQLite aux lettres de motivation.

use crate::modules::lettres::dtos::NouvelleLettre;
use crate::modules::lettres::model::LettreMotivation;
use crate::shared::db::SqlitePool;
use crate::shared::error::{AppError, AppResult};
use crate::shared::sqlite::{connexion, maintenant_iso, traduire_erreur, uuid_colonne};

/// Contrat d'accès à la bibliothèque de lettres.
pub trait LettreRepository: Send + Sync {
    fn create(&self, input: &NouvelleLettre) -> AppResult<LettreMotivation>;
    fn list(&self) -> AppResult<Vec<LettreMotivation>>;
    fn get(&self, id: uuid::Uuid) -> AppResult<LettreMotivation>;
    fn delete(&self, id: uuid::Uuid) -> AppResult<()>;
}

/// Dépôt SQLite de production.
pub struct SqliteLettreRepository {
    pool: SqlitePool,
}

impl SqliteLettreRepository {
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

const COLONNES: &str = "id, name, company, job_title, tone, length, content, created_at";

fn row_to_letter(row: &rusqlite::Row) -> rusqlite::Result<LettreMotivation> {
    Ok(LettreMotivation {
        id: uuid_colonne(row, 0)?,
        name: row.get(1)?,
        company: row.get(2)?,
        job_title: row.get(3)?,
        tone: row.get(4)?,
        length: row.get(5)?,
        content: row.get(6)?,
        created_at: row.get(7)?,
    })
}

impl LettreRepository for SqliteLettreRepository {
    fn create(&self, input: &NouvelleLettre) -> AppResult<LettreMotivation> {
        let conn = connexion(&self.pool)?;
        let id = uuid::Uuid::new_v4();
        let created_at = maintenant_iso();
        conn.execute(
            "INSERT INTO lettres_motivation (id, name, company, job_title, tone, length, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                id.to_string(),
                input.name,
                input.company,
                input.job_title,
                input.tone,
                input.length,
                input.content,
                created_at,
            ],
        )
        .map_err(|error| traduire_erreur(error, "lettre de motivation"))?;
        self.get(id)
    }

    fn list(&self) -> AppResult<Vec<LettreMotivation>> {
        let conn = connexion(&self.pool)?;
        let mut query = conn
            .prepare(&format!(
                "SELECT {COLONNES} FROM lettres_motivation ORDER BY created_at DESC, rowid DESC"
            ))
            .map_err(|error| traduire_erreur(error, "lettres de motivation"))?;
        let rows = query
            .query_map([], row_to_letter)
            .map_err(|error| traduire_erreur(error, "lettres de motivation"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| traduire_erreur(error, "lettres de motivation"))
    }

    fn get(&self, id: uuid::Uuid) -> AppResult<LettreMotivation> {
        let conn = connexion(&self.pool)?;
        conn.query_row(
            &format!("SELECT {COLONNES} FROM lettres_motivation WHERE id = ?1"),
            [id.to_string()],
            row_to_letter,
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("lettre de motivation {id}"))
            }
            other => traduire_erreur(other, "lettre de motivation"),
        })
    }

    fn delete(&self, id: uuid::Uuid) -> AppResult<()> {
        let conn = connexion(&self.pool)?;
        conn.execute(
            "DELETE FROM lettres_motivation WHERE id = ?1",
            [id.to_string()],
        )
        .map_err(|error| traduire_erreur(error, "lettre de motivation"))?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/repository/mod.rs"]
mod tests;
