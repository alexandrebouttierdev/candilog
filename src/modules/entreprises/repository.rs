//! Accès aux entreprises (base locale `SQLite`).

use crate::modules::entreprises::model::{Entreprise, NouvelleEntreprise};
use crate::shared::db::SqlitePool;
use crate::shared::error::{AppError, AppResult};
use crate::shared::sqlite::{
    connexion, maintenant_iso, traduire_contrainte, traduire_erreur, uuid_colonne,
};

/// Contrat d'accès aux entreprises.
pub trait EntrepriseRepository: Send + Sync {
    /// Liste les entreprises (triées par nom).
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<Entreprise>>;
    /// Crée une entreprise.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si l'insertion échoue.
    fn create(&self, input: &NouvelleEntreprise) -> AppResult<Entreprise>;
    /// Met à jour une entreprise.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn update(&self, id: uuid::Uuid, input: &NouvelleEntreprise) -> AppResult<Entreprise>;
    /// Supprime une entreprise.
    ///
    /// # Errors
    /// `AppError::Validation` si des candidatures y sont rattachées.
    fn delete(&self, id: uuid::Uuid) -> AppResult<()>;
}

/// Implémentation `SQLite` du dépôt d'entreprises.
pub struct SqliteEntrepriseRepository {
    pool: SqlitePool,
}

impl SqliteEntrepriseRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Colonnes lues par [`ligne_vers_entreprise`], dans l'ordre.
const COLONNES: &str =
    "id, nom, secteur, type, site_web, ville, adresse, notes, created_at, updated_at";

/// Convertit une ligne `SQLite` en entreprise du domaine.
fn ligne_vers_entreprise(row: &rusqlite::Row) -> rusqlite::Result<Entreprise> {
    Ok(Entreprise {
        id: uuid_colonne(row, 0)?,
        nom: row.get(1)?,
        secteur: row.get(2)?,
        type_: row.get(3)?,
        site_web: row.get(4)?,
        ville: row.get(5)?,
        adresse: row.get(6)?,
        notes: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

impl EntrepriseRepository for SqliteEntrepriseRepository {
    fn list(&self) -> AppResult<Vec<Entreprise>> {
        let conn = connexion(&self.pool)?;
        let mut requete = conn
            .prepare(&format!(
                "SELECT {COLONNES} FROM entreprises ORDER BY nom COLLATE NOCASE ASC"
            ))
            .map_err(|e| traduire_erreur(e, "entreprises"))?;
        let lignes = requete
            .query_map([], ligne_vers_entreprise)
            .map_err(|e| traduire_erreur(e, "entreprises"))?;
        let mut entreprises = Vec::new();
        for ligne in lignes {
            entreprises.push(ligne.map_err(|e| traduire_erreur(e, "entreprises"))?);
        }
        Ok(entreprises)
    }

    fn create(&self, input: &NouvelleEntreprise) -> AppResult<Entreprise> {
        let conn = connexion(&self.pool)?;
        let id = uuid::Uuid::new_v4();
        let maintenant = maintenant_iso();
        conn.execute(
            "INSERT INTO entreprises (id, nom, secteur, type, site_web, ville, adresse, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            rusqlite::params![
                id.to_string(), input.nom, input.secteur, input.type_, input.site_web,
                input.ville, input.adresse, input.notes, maintenant
            ],
        )
        .map_err(|e| traduire_erreur(e, "entreprise invalide"))?;
        conn.query_row(
            &format!("SELECT {COLONNES} FROM entreprises WHERE id = ?1"),
            [id.to_string()],
            ligne_vers_entreprise,
        )
        .map_err(|e| traduire_erreur(e, "entreprise"))
    }

    fn update(&self, id: uuid::Uuid, input: &NouvelleEntreprise) -> AppResult<Entreprise> {
        let conn = connexion(&self.pool)?;
        let modifiees = conn
            .execute(
                "UPDATE entreprises SET nom = ?2, secteur = ?3, type = ?4, site_web = ?5,
                    ville = ?6, adresse = ?7, notes = ?8, updated_at = ?9
                 WHERE id = ?1",
                rusqlite::params![
                    id.to_string(),
                    input.nom,
                    input.secteur,
                    input.type_,
                    input.site_web,
                    input.ville,
                    input.adresse,
                    input.notes,
                    maintenant_iso()
                ],
            )
            .map_err(|e| traduire_erreur(e, "entreprise invalide"))?;
        if modifiees == 0 {
            return Err(AppError::NotFound(format!("entreprise {id}")));
        }
        conn.query_row(
            &format!("SELECT {COLONNES} FROM entreprises WHERE id = ?1"),
            [id.to_string()],
            ligne_vers_entreprise,
        )
        .map_err(|e| traduire_erreur(e, "entreprise"))
    }

    fn delete(&self, id: uuid::Uuid) -> AppResult<()> {
        let conn = connexion(&self.pool)?;
        conn.execute("DELETE FROM entreprises WHERE id = ?1", [id.to_string()])
            .map_err(|e| {
                traduire_contrainte(
                    e,
                    "Suppression impossible : des candidatures sont liées à cette entreprise",
                    "entreprise",
                )
            })?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/repository/mod.rs"]
mod tests;
