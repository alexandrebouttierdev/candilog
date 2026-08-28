//! Dépôt `SQLite` des relances.

use crate::core::database::helpers::{
    connexion, maintenant_iso, traduire_contrainte, traduire_erreur, uuid_colonne,
};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::relances::domain::{NouvelleRelance, Relance, RelanceRepository};
use uuid::Uuid;

/// Implémentation `SQLite` du dépôt de relances.
pub struct SqliteRelanceRepository {
    pool: SqlitePool,
}

impl SqliteRelanceRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Colonnes lues par [`ligne_vers_relance`], dans l'ordre.
const COLONNES: &str = "r.id, r.candidature_id, c.poste, e.nom, r.date_relance, r.type, \
                        r.notes, r.created_at";

/// Source des colonnes : jointures pour afficher le poste et l'entreprise au calendrier.
const DEPUIS: &str = "FROM relances r \
                      LEFT JOIN candidatures c ON c.id = r.candidature_id \
                      LEFT JOIN entreprises e ON e.id = c.entreprise_id";

/// Convertit une ligne `SQLite` en relance du domaine.
fn ligne_vers_relance(row: &rusqlite::Row) -> rusqlite::Result<Relance> {
    Ok(Relance {
        id: uuid_colonne(row, 0)?,
        candidature_id: uuid_colonne(row, 1)?,
        candidature_poste: row.get(2)?,
        entreprise_nom: row.get(3)?,
        date_relance: row.get(4)?,
        type_relance: row.get(5)?,
        notes: row.get(6)?,
        created_at: row.get(7)?,
    })
}

/// Lit une requête de relances jusqu'au bout.
fn collecter(
    conn: &rusqlite::Connection,
    sql: &str,
    params: &[&dyn rusqlite::ToSql],
) -> AppResult<Vec<Relance>> {
    let mut requete = conn
        .prepare(sql)
        .map_err(|e| traduire_erreur(e, "relances"))?;
    let lignes = requete
        .query_map(params, ligne_vers_relance)
        .map_err(|e| traduire_erreur(e, "relances"))?;
    let mut items = Vec::new();
    for ligne in lignes {
        items.push(ligne.map_err(|e| traduire_erreur(e, "relances"))?);
    }
    Ok(items)
}

impl RelanceRepository for SqliteRelanceRepository {
    fn list(&self) -> AppResult<Vec<Relance>> {
        let conn = connexion(&self.pool)?;
        collecter(
            &conn,
            &format!("SELECT {COLONNES} {DEPUIS} ORDER BY r.date_relance DESC"),
            &[],
        )
    }

    fn list_between(&self, from: &str, to: &str) -> AppResult<Vec<Relance>> {
        let conn = connexion(&self.pool)?;
        collecter(
            &conn,
            &format!(
                "SELECT {COLONNES} {DEPUIS} \
                 WHERE r.date_relance >= ?1 AND r.date_relance <= ?2 \
                 ORDER BY r.date_relance ASC"
            ),
            &[&from, &to],
        )
    }

    fn create(&self, input: &NouvelleRelance) -> AppResult<Relance> {
        let conn = connexion(&self.pool)?;
        let id = Uuid::new_v4();
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
        self.obtenir(id)
    }

    fn update(&self, id: Uuid, input: &NouvelleRelance) -> AppResult<Relance> {
        let conn = connexion(&self.pool)?;
        let modifiees = conn
            .execute(
                "UPDATE relances SET candidature_id = ?2, date_relance = ?3, type = ?4,
                    notes = ?5
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
        self.obtenir(id)
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = connexion(&self.pool)?;
        conn.execute("DELETE FROM relances WHERE id = ?1", [id.to_string()])
            .map_err(|e| traduire_erreur(e, "relance"))?;
        Ok(())
    }
}

impl SqliteRelanceRepository {
    /// Relit une relance après écriture.
    fn obtenir(&self, id: Uuid) -> AppResult<Relance> {
        let conn = connexion(&self.pool)?;
        collecter(
            &conn,
            &format!("SELECT {COLONNES} {DEPUIS} WHERE r.id = ?1"),
            &[&id.to_string()],
        )?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound(format!("relance {id}")))
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
