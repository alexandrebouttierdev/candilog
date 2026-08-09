//! Accès aux candidatures (base locale `SQLite`).

use crate::modules::candidatures::model::{Candidature, NouvelleCandidature, StatutCandidature};
use crate::shared::db::SqlitePool;
use crate::shared::error::{AppError, AppResult};
use crate::shared::sqlite::{
    connexion, enum_depuis_texte, maintenant_iso, texte_depuis_enum, traduire_contrainte,
    traduire_erreur, uuid_colonne, uuid_colonne_opt,
};
use uuid::Uuid;

/// Contrat d'accès aux candidatures.
pub trait CandidatureRepository: Send + Sync {
    /// Crée une candidature.
    ///
    /// # Errors
    /// `AppError::Validation` si l'entreprise liée est introuvable ; sinon `AppError::Database`.
    fn create(&self, input: &NouvelleCandidature) -> AppResult<Candidature>;
    /// Liste les candidatures (les plus récentes d'abord), avec le nom de l'entreprise liée.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<Candidature>>;
    /// Met à jour tous les champs éditables d'une candidature.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu ; `AppError::Validation` si l'entreprise
    /// liée est introuvable.
    fn update(&self, id: Uuid, input: &NouvelleCandidature) -> AppResult<Candidature>;
    /// Met à jour uniquement le statut (cible du drag & drop).
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn update_statut(&self, id: Uuid, statut: StatutCandidature) -> AppResult<Candidature>;
    /// Supprime une candidature par identifiant (et ses relances/entretiens en cascade).
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la suppression échoue.
    fn delete(&self, id: Uuid) -> AppResult<()>;
}

/// Implémentation `SQLite` du dépôt de candidatures.
pub struct SqliteCandidatureRepository {
    pool: SqlitePool,
}

impl SqliteCandidatureRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Colonnes lues par [`ligne_vers_candidature`], dans l'ordre (avec le nom d'entreprise joint).
const COLONNES: &str = "c.id, c.poste, c.entreprise_id, e.nom, c.contact_id, c.type_contrat,
    c.statut, c.date_envoi, c.lien_offre, c.notes, c.created_at, c.updated_at";

/// Source de la requête : jointure sur `entreprises` pour exposer `entreprise_nom`.
const DEPUIS: &str = "FROM candidatures c LEFT JOIN entreprises e ON e.id = c.entreprise_id";

/// Convertit une ligne `SQLite` (avec jointure) en candidature du domaine.
///
/// Renvoie `AppResult` (et non `rusqlite::Result`) car `type_contrat` et `statut` sont des enums
/// serde dont la conversion depuis le `TEXT` stocké peut échouer.
fn ligne_vers_candidature(row: &rusqlite::Row) -> AppResult<Candidature> {
    let type_contrat: String = row.get(5).map_err(|e| traduire_erreur(e, "candidature"))?;
    let statut: String = row.get(6).map_err(|e| traduire_erreur(e, "candidature"))?;
    Ok(Candidature {
        id: uuid_colonne(row, 0).map_err(|e| traduire_erreur(e, "candidature"))?,
        poste: row.get(1).map_err(|e| traduire_erreur(e, "candidature"))?,
        entreprise_id: uuid_colonne(row, 2).map_err(|e| traduire_erreur(e, "candidature"))?,
        entreprise_nom: row.get(3).map_err(|e| traduire_erreur(e, "candidature"))?,
        contact_id: uuid_colonne_opt(row, 4).map_err(|e| traduire_erreur(e, "candidature"))?,
        type_contrat: enum_depuis_texte(&type_contrat)?,
        statut: enum_depuis_texte(&statut)?,
        date_envoi: row.get(7).map_err(|e| traduire_erreur(e, "candidature"))?,
        lien_offre: row.get(8).map_err(|e| traduire_erreur(e, "candidature"))?,
        notes: row.get(9).map_err(|e| traduire_erreur(e, "candidature"))?,
        created_at: row.get(10).map_err(|e| traduire_erreur(e, "candidature"))?,
        updated_at: row.get(11).map_err(|e| traduire_erreur(e, "candidature"))?,
    })
}

impl CandidatureRepository for SqliteCandidatureRepository {
    fn create(&self, input: &NouvelleCandidature) -> AppResult<Candidature> {
        let conn = connexion(&self.pool)?;
        let id = Uuid::new_v4();
        let maintenant = maintenant_iso();
        conn.execute(
            "INSERT INTO candidatures (id, entreprise_id, contact_id, poste, type_contrat, statut,
                date_envoi, lien_offre, notes, created_at, updated_at)
             VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            rusqlite::params![
                id.to_string(),
                input.entreprise_id.to_string(),
                input.poste,
                texte_depuis_enum(&input.type_contrat)?,
                texte_depuis_enum(&input.statut)?,
                input.date_envoi,
                input.lien_offre,
                input.notes,
                maintenant
            ],
        )
        .map_err(|e| {
            traduire_contrainte(
                e,
                "L'entreprise liée à cette candidature est introuvable",
                "candidature",
            )
        })?;
        conn.query_row(
            &format!("SELECT {COLONNES} {DEPUIS} WHERE c.id = ?1"),
            [id.to_string()],
            |row| Ok(ligne_vers_candidature(row)),
        )
        .map_err(|e| traduire_erreur(e, "candidature"))?
    }

    fn list(&self) -> AppResult<Vec<Candidature>> {
        let conn = connexion(&self.pool)?;
        let mut requete = conn
            .prepare(&format!(
                "SELECT {COLONNES} {DEPUIS} ORDER BY c.date_envoi DESC, c.created_at DESC"
            ))
            .map_err(|e| traduire_erreur(e, "candidatures"))?;
        let mut lignes = requete
            .query([])
            .map_err(|e| traduire_erreur(e, "candidatures"))?;
        let mut candidatures = Vec::new();
        while let Some(row) = lignes
            .next()
            .map_err(|e| traduire_erreur(e, "candidatures"))?
        {
            candidatures.push(ligne_vers_candidature(row)?);
        }
        Ok(candidatures)
    }

    fn update(&self, id: Uuid, input: &NouvelleCandidature) -> AppResult<Candidature> {
        let conn = connexion(&self.pool)?;
        let modifiees = conn
            .execute(
                "UPDATE candidatures SET entreprise_id = ?2, poste = ?3, type_contrat = ?4, statut = ?5,
                    date_envoi = ?6, lien_offre = ?7, notes = ?8, updated_at = ?9
                 WHERE id = ?1",
                rusqlite::params![
                    id.to_string(),
                    input.entreprise_id.to_string(),
                    input.poste,
                    texte_depuis_enum(&input.type_contrat)?,
                    texte_depuis_enum(&input.statut)?,
                    input.date_envoi,
                    input.lien_offre,
                    input.notes,
                    maintenant_iso()
                ],
            )
            .map_err(|e| {
                traduire_contrainte(
                    e,
                    "L'entreprise liée à cette candidature est introuvable",
                    "candidature",
                )
            })?;
        if modifiees == 0 {
            return Err(AppError::NotFound(format!("candidature {id}")));
        }
        conn.query_row(
            &format!("SELECT {COLONNES} {DEPUIS} WHERE c.id = ?1"),
            [id.to_string()],
            |row| Ok(ligne_vers_candidature(row)),
        )
        .map_err(|e| traduire_erreur(e, "candidature"))?
    }

    fn update_statut(&self, id: Uuid, statut: StatutCandidature) -> AppResult<Candidature> {
        let conn = connexion(&self.pool)?;
        let modifiees = conn
            .execute(
                "UPDATE candidatures SET statut = ?2, updated_at = ?3 WHERE id = ?1",
                rusqlite::params![
                    id.to_string(),
                    texte_depuis_enum(&statut)?,
                    maintenant_iso()
                ],
            )
            .map_err(|e| traduire_erreur(e, "candidature invalide"))?;
        if modifiees == 0 {
            return Err(AppError::NotFound(format!("candidature {id}")));
        }
        conn.query_row(
            &format!("SELECT {COLONNES} {DEPUIS} WHERE c.id = ?1"),
            [id.to_string()],
            |row| Ok(ligne_vers_candidature(row)),
        )
        .map_err(|e| traduire_erreur(e, "candidature"))?
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = connexion(&self.pool)?;
        conn.execute("DELETE FROM candidatures WHERE id = ?1", [id.to_string()])
            .map_err(|e| traduire_erreur(e, "candidature"))?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/repository/mod.rs"]
mod tests;
