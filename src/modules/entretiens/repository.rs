//! Accès aux entretiens (base locale `SQLite`).

use crate::modules::entretiens::model::{Entretien, NouvelEntretien};
use crate::shared::db::SqlitePool;
use crate::shared::error::{AppError, AppResult};
use crate::shared::sqlite::{
    connexion, enum_depuis_texte, maintenant_iso, texte_depuis_enum, traduire_contrainte,
    traduire_erreur, uuid_colonne, uuid_colonne_opt,
};
use crate::shared::types::AnalyseEntretien;
use uuid::Uuid;

/// Contrat d'accès aux entretiens.
pub trait EntretienRepository: Send + Sync {
    /// Liste les entretiens (triés par date croissante).
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<Entretien>>;
    /// Liste uniquement les entretiens compris dans une fenêtre ISO inclusive.
    fn list_between(&self, from: &str, to: &str) -> AppResult<Vec<Entretien>> {
        Ok(self
            .list()?
            .into_iter()
            .filter(|item| {
                item.date_entretien.as_str() >= from && item.date_entretien.as_str() <= to
            })
            .collect())
    }
    /// Crée un entretien.
    ///
    /// # Errors
    /// `AppError::Validation` si la candidature ou le contact lié est introuvable ; sinon
    /// `AppError::Database`.
    fn create(&self, input: &NouvelEntretien) -> AppResult<Entretien>;
    /// Met à jour tous les champs éditables d'un entretien.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu ; `AppError::Validation` si la
    /// candidature ou le contact lié est introuvable.
    fn update(&self, id: Uuid, input: &NouvelEntretien) -> AppResult<Entretien>;
    /// Crée ou modifie un entretien et marque sa candidature en entretien atomiquement.
    fn save_and_mark_candidate(
        &self,
        id: Option<Uuid>,
        input: &NouvelEntretien,
    ) -> AppResult<Entretien> {
        id.map_or_else(|| self.create(input), |id| self.update(id, input))
    }
    /// Supprime un entretien par identifiant.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la suppression échoue.
    fn delete(&self, id: Uuid) -> AppResult<()>;
    /// Récupère un entretien par son identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'entretien n'existe pas ; sinon `AppError::Database`.
    fn get(&self, id: Uuid) -> AppResult<Entretien>;
    /// Persiste l'analyse `IA` du compte rendu sur l'entretien.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu ; sinon `AppError::Database`.
    fn enregistrer_analyse(&self, id: Uuid, analyse: &AnalyseEntretien) -> AppResult<()>;
}

/// Implémentation `SQLite` du dépôt d'entretiens.
pub struct SqliteEntretienRepository {
    pool: SqlitePool,
}

impl SqliteEntretienRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Colonnes lues par [`ligne_vers_entretien`], dans l'ordre.
const COLONNES: &str = "id, candidature_id, contact_id, date_entretien, type, lieu, notes,
    compte_rendu, analyse_ia, created_at, updated_at";

/// Convertit une ligne `SQLite` en entretien du domaine.
///
/// Renvoie `AppResult` (et non `rusqlite::Result`) car `type` est un enum serde et `analyse_ia`
/// un `TEXT` `JSON` dont la conversion depuis la colonne stockée peut échouer.
fn ligne_vers_entretien(row: &rusqlite::Row) -> AppResult<Entretien> {
    let type_texte: String = row.get(4).map_err(|e| traduire_erreur(e, "entretien"))?;
    let analyse_json: Option<String> = row.get(8).map_err(|e| traduire_erreur(e, "entretien"))?;
    let analyse_ia = match analyse_json {
        None => None,
        Some(json) => {
            Some(serde_json::from_str(&json).map_err(|e| AppError::Serialization(e.to_string()))?)
        }
    };
    Ok(Entretien {
        id: uuid_colonne(row, 0).map_err(|e| traduire_erreur(e, "entretien"))?,
        candidature_id: uuid_colonne(row, 1).map_err(|e| traduire_erreur(e, "entretien"))?,
        contact_id: uuid_colonne_opt(row, 2).map_err(|e| traduire_erreur(e, "entretien"))?,
        date_entretien: row.get(3).map_err(|e| traduire_erreur(e, "entretien"))?,
        type_entretien: enum_depuis_texte(&type_texte)?,
        lieu: row.get(5).map_err(|e| traduire_erreur(e, "entretien"))?,
        notes: row.get(6).map_err(|e| traduire_erreur(e, "entretien"))?,
        compte_rendu: row.get(7).map_err(|e| traduire_erreur(e, "entretien"))?,
        analyse_ia,
        created_at: row.get(9).map_err(|e| traduire_erreur(e, "entretien"))?,
        updated_at: row.get(10).map_err(|e| traduire_erreur(e, "entretien"))?,
    })
}

impl EntretienRepository for SqliteEntretienRepository {
    fn list(&self) -> AppResult<Vec<Entretien>> {
        let conn = connexion(&self.pool)?;
        let mut requete = conn
            .prepare(&format!(
                "SELECT {COLONNES} FROM entretiens ORDER BY date_entretien ASC"
            ))
            .map_err(|e| traduire_erreur(e, "entretiens"))?;
        let mut lignes = requete
            .query([])
            .map_err(|e| traduire_erreur(e, "entretiens"))?;
        let mut entretiens = Vec::new();
        while let Some(row) = lignes
            .next()
            .map_err(|e| traduire_erreur(e, "entretiens"))?
        {
            entretiens.push(ligne_vers_entretien(row)?);
        }
        Ok(entretiens)
    }

    fn list_between(&self, from: &str, to: &str) -> AppResult<Vec<Entretien>> {
        let conn = connexion(&self.pool)?;
        let mut statement = conn
            .prepare(&format!(
                "SELECT {COLONNES} FROM entretiens WHERE date_entretien >= ?1 AND date_entretien <= ?2 ORDER BY date_entretien ASC"
            ))
            .map_err(|e| traduire_erreur(e, "entretiens"))?;
        let mut rows = statement
            .query([from, to])
            .map_err(|e| traduire_erreur(e, "entretiens"))?;
        let mut items = Vec::new();
        while let Some(row) = rows.next().map_err(|e| traduire_erreur(e, "entretiens"))? {
            items.push(ligne_vers_entretien(row)?);
        }
        Ok(items)
    }

    fn create(&self, input: &NouvelEntretien) -> AppResult<Entretien> {
        let conn = connexion(&self.pool)?;
        let id = Uuid::new_v4();
        let maintenant = maintenant_iso();
        conn.execute(
            "INSERT INTO entretiens (id, candidature_id, contact_id, date_entretien, type, lieu, notes,
                compte_rendu, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            rusqlite::params![
                id.to_string(),
                input.candidature_id.to_string(),
                input.contact_id.map(|c| c.to_string()),
                input.date_entretien,
                texte_depuis_enum(&input.type_entretien)?,
                input.lieu,
                input.notes,
                input.compte_rendu,
                maintenant,
            ],
        )
        .map_err(|e| {
            traduire_contrainte(
                e,
                "La candidature ou le contact lié à cet entretien est introuvable",
                "entretien",
            )
        })?;
        conn.query_row(
            &format!("SELECT {COLONNES} FROM entretiens WHERE id = ?1"),
            [id.to_string()],
            |row| Ok(ligne_vers_entretien(row)),
        )
        .map_err(|e| traduire_erreur(e, "entretien"))?
    }

    fn update(&self, id: Uuid, input: &NouvelEntretien) -> AppResult<Entretien> {
        let conn = connexion(&self.pool)?;
        let modifiees = conn
            .execute(
                "UPDATE entretiens SET candidature_id = ?2, contact_id = ?3, date_entretien = ?4, type = ?5,
                    lieu = ?6, notes = ?7, compte_rendu = ?8, updated_at = ?9
                 WHERE id = ?1",
                rusqlite::params![
                    id.to_string(),
                    input.candidature_id.to_string(),
                    input.contact_id.map(|c| c.to_string()),
                    input.date_entretien,
                    texte_depuis_enum(&input.type_entretien)?,
                    input.lieu,
                    input.notes,
                    input.compte_rendu,
                    maintenant_iso(),
                ],
            )
            .map_err(|e| {
                traduire_contrainte(
                    e,
                    "La candidature ou le contact lié à cet entretien est introuvable",
                    "entretien",
                )
            })?;
        if modifiees == 0 {
            return Err(AppError::NotFound(format!("entretien {id}")));
        }
        conn.query_row(
            &format!("SELECT {COLONNES} FROM entretiens WHERE id = ?1"),
            [id.to_string()],
            |row| Ok(ligne_vers_entretien(row)),
        )
        .map_err(|e| traduire_erreur(e, "entretien"))?
    }

    fn save_and_mark_candidate(
        &self,
        id: Option<Uuid>,
        input: &NouvelEntretien,
    ) -> AppResult<Entretien> {
        let mut conn = connexion(&self.pool)?;
        let entretien_id = id.unwrap_or_else(Uuid::new_v4);
        let maintenant = maintenant_iso();
        let transaction = conn.transaction()?;
        let type_entretien = texte_depuis_enum(&input.type_entretien)?;
        let affected =
            if id.is_some() {
                transaction.execute(
                "UPDATE entretiens SET candidature_id = ?2, contact_id = ?3, date_entretien = ?4,
                    type = ?5, lieu = ?6, notes = ?7, compte_rendu = ?8, updated_at = ?9
                 WHERE id = ?1",
                rusqlite::params![
                    entretien_id.to_string(), input.candidature_id.to_string(),
                    input.contact_id.map(|value| value.to_string()), input.date_entretien,
                    type_entretien, input.lieu, input.notes, input.compte_rendu, maintenant,
                ],
            )?
            } else {
                transaction.execute(
                    "INSERT INTO entretiens (id, candidature_id, contact_id, date_entretien, type,
                    lieu, notes, compte_rendu, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
                    rusqlite::params![
                        entretien_id.to_string(),
                        input.candidature_id.to_string(),
                        input.contact_id.map(|value| value.to_string()),
                        input.date_entretien,
                        type_entretien,
                        input.lieu,
                        input.notes,
                        input.compte_rendu,
                        maintenant,
                    ],
                )?
            };
        if affected == 0 {
            return Err(AppError::NotFound(format!("entretien {entretien_id}")));
        }

        let interview_status = crate::shared::sqlite::texte_depuis_enum(
            &crate::modules::candidatures::model::StatutCandidature::Entretien,
        )?;
        let previous_status: String = transaction
            .query_row(
                "SELECT statut FROM candidatures WHERE id = ?1",
                [input.candidature_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => AppError::Validation(
                    "La candidature liée à cet entretien est introuvable".into(),
                ),
                other => traduire_erreur(other, "candidature"),
            })?;
        transaction.execute(
            "UPDATE candidatures SET statut = ?2, updated_at = ?3 WHERE id = ?1",
            rusqlite::params![
                input.candidature_id.to_string(),
                interview_status,
                maintenant
            ],
        )?;
        if previous_status != interview_status {
            transaction.execute(
                "INSERT INTO statut_history (id, candidature_id, statut, changed_at)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    Uuid::new_v4().to_string(),
                    input.candidature_id.to_string(),
                    interview_status,
                    maintenant,
                ],
            )?;
        }
        transaction.commit()?;
        self.get(entretien_id)
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = connexion(&self.pool)?;
        conn.execute("DELETE FROM entretiens WHERE id = ?1", [id.to_string()])
            .map_err(|e| traduire_erreur(e, "entretien"))?;
        Ok(())
    }

    fn get(&self, id: Uuid) -> AppResult<Entretien> {
        let conn = connexion(&self.pool)?;
        conn.query_row(
            &format!("SELECT {COLONNES} FROM entretiens WHERE id = ?1"),
            [id.to_string()],
            |row| Ok(ligne_vers_entretien(row)),
        )
        .map_err(|e| traduire_erreur(e, "entretien"))?
    }

    fn enregistrer_analyse(&self, id: Uuid, analyse: &AnalyseEntretien) -> AppResult<()> {
        let conn = connexion(&self.pool)?;
        let json =
            serde_json::to_string(analyse).map_err(|e| AppError::Serialization(e.to_string()))?;
        let modifiees = conn
            .execute(
                "UPDATE entretiens SET analyse_ia = ?2, updated_at = ?3 WHERE id = ?1",
                rusqlite::params![id.to_string(), json, maintenant_iso()],
            )
            .map_err(|e| traduire_erreur(e, "entretien"))?;
        if modifiees == 0 {
            return Err(AppError::NotFound(format!("entretien {id}")));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/repository/mod.rs"]
mod tests;
