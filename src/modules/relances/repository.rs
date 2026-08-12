//! Accès aux relances (base locale `SQLite`).

use crate::modules::relances::model::{NouvelleRelance, Relance};
use crate::shared::db::SqlitePool;
use crate::shared::error::{AppError, AppResult};
use crate::shared::sqlite::{
    connexion, maintenant_iso, traduire_contrainte, traduire_erreur, uuid_colonne,
};

/// Contrat d'accès aux relances.
pub trait RelanceRepository: Send + Sync {
    /// Liste les relances (triées par date croissante).
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<Relance>>;
    /// Liste uniquement les relances comprises dans une fenêtre ISO inclusive.
    fn list_between(&self, from: &str, to: &str) -> AppResult<Vec<Relance>> {
        Ok(self
            .list()?
            .into_iter()
            .filter(|item| item.date_relance.as_str() >= from && item.date_relance.as_str() <= to)
            .collect())
    }
    /// Crée une relance.
    ///
    /// # Errors
    /// Retourne `AppError::Validation` si la candidature liée est introuvable, sinon
    /// `AppError::Database` si l'insertion échoue.
    fn create(&self, input: &NouvelleRelance) -> AppResult<Relance>;
    /// Met à jour une relance.
    ///
    /// # Errors
    /// Retourne `AppError::NotFound` si l'identifiant est inconnu, `AppError::Validation` si la
    /// candidature liée est introuvable, sinon `AppError::Database` si la mise à jour échoue.
    fn update(&self, id: uuid::Uuid, input: &NouvelleRelance) -> AppResult<Relance>;
    /// Supprime une relance.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la suppression échoue.
    fn delete(&self, id: uuid::Uuid) -> AppResult<()>;
}

/// Implémentation `SQLite` du dépôt de relances.
pub struct SqliteRelanceRepository {
    pool: SqlitePool,
}

impl SqliteRelanceRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Colonnes lues par [`ligne_vers_relance`], dans l'ordre.
const COLONNES: &str = "id, candidature_id, date_relance, type, notes, created_at";

/// Convertit une ligne `SQLite` en relance du domaine.
fn ligne_vers_relance(row: &rusqlite::Row) -> rusqlite::Result<Relance> {
    Ok(Relance {
        id: uuid_colonne(row, 0)?,
        candidature_id: uuid_colonne(row, 1)?,
        date_relance: row.get(2)?,
        type_relance: row.get(3)?,
        notes: row.get(4)?,
        created_at: row.get(5)?,
    })
}

impl RelanceRepository for SqliteRelanceRepository {
    fn list(&self) -> AppResult<Vec<Relance>> {
        let conn = connexion(&self.pool)?;
        let mut requete = conn
            .prepare(&format!(
                "SELECT {COLONNES} FROM relances ORDER BY date_relance ASC"
            ))
            .map_err(|e| traduire_erreur(e, "relances"))?;
        let lignes = requete
            .query_map([], ligne_vers_relance)
            .map_err(|e| traduire_erreur(e, "relances"))?;
        let mut relances = Vec::new();
        for ligne in lignes {
            relances.push(ligne.map_err(|e| traduire_erreur(e, "relances"))?);
        }
        Ok(relances)
    }

    fn list_between(&self, from: &str, to: &str) -> AppResult<Vec<Relance>> {
        let conn = connexion(&self.pool)?;
        let mut statement = conn
            .prepare(&format!(
                "SELECT {COLONNES} FROM relances
                 WHERE date_relance >= ?1 AND date_relance <= ?2 ORDER BY date_relance ASC"
            ))
            .map_err(|error| traduire_erreur(error, "relances"))?;
        let rows = statement
            .query_map([from, to], ligne_vers_relance)
            .map_err(|error| traduire_erreur(error, "relances"))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|error| traduire_erreur(error, "relances"))?);
        }
        Ok(items)
    }

    fn create(&self, input: &NouvelleRelance) -> AppResult<Relance> {
        let conn = connexion(&self.pool)?;
        let id = uuid::Uuid::new_v4();
        conn.execute(
            "INSERT INTO relances (id, candidature_id, date_relance, type, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                id.to_string(),
                input.candidature_id.to_string(),
                input.date_relance,
                input.type_relance,
                input.notes,
                maintenant_iso(),
            ],
        )
        .map_err(|e| {
            traduire_contrainte(
                e,
                "La candidature liée à cette relance est introuvable",
                "relance",
            )
        })?;
        conn.query_row(
            &format!("SELECT {COLONNES} FROM relances WHERE id = ?1"),
            [id.to_string()],
            ligne_vers_relance,
        )
        .map_err(|e| traduire_erreur(e, "relance"))
    }

    fn update(&self, id: uuid::Uuid, input: &NouvelleRelance) -> AppResult<Relance> {
        let conn = connexion(&self.pool)?;
        let modifiees = conn
            .execute(
                "UPDATE relances SET candidature_id = ?2, date_relance = ?3, type = ?4, notes = ?5
                 WHERE id = ?1",
                rusqlite::params![
                    id.to_string(),
                    input.candidature_id.to_string(),
                    input.date_relance,
                    input.type_relance,
                    input.notes,
                ],
            )
            .map_err(|e| {
                traduire_contrainte(
                    e,
                    "La candidature liée à cette relance est introuvable",
                    "relance",
                )
            })?;
        if modifiees == 0 {
            return Err(AppError::NotFound(format!("relance {id}")));
        }
        conn.query_row(
            &format!("SELECT {COLONNES} FROM relances WHERE id = ?1"),
            [id.to_string()],
            ligne_vers_relance,
        )
        .map_err(|e| traduire_erreur(e, "relance"))
    }

    fn delete(&self, id: uuid::Uuid) -> AppResult<()> {
        let conn = connexion(&self.pool)?;
        conn.execute("DELETE FROM relances WHERE id = ?1", [id.to_string()])
            .map_err(|e| traduire_erreur(e, "relance"))?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/repository/mod.rs"]
mod tests;
