//! Dépôt `SQLite` des entreprises.

use crate::core::database::helpers::{
    connexion, maintenant_iso, traduire_contrainte, traduire_erreur, uuid_colonne, uuid_colonne_opt,
};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::Page;
use crate::features::entreprises::domain::{Entreprise, EntrepriseRepository, NouvelleEntreprise};

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
const COLONNES: &str = "id, nom, secteur_id, secteur, type, site_web, ville, adresse, notes, \
                        created_at, updated_at";

/// Convertit une ligne `SQLite` en entreprise du domaine.
fn ligne_vers_entreprise(row: &rusqlite::Row) -> rusqlite::Result<Entreprise> {
    Ok(Entreprise {
        id: uuid_colonne(row, 0)?,
        nom: row.get(1)?,
        secteur_id: uuid_colonne_opt(row, 2)?,
        secteur: row.get(3)?,
        type_: row.get(4)?,
        site_web: row.get(5)?,
        ville: row.get(6)?,
        adresse: row.get(7)?,
        notes: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
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

    fn get(&self, id: uuid::Uuid) -> AppResult<Entreprise> {
        let conn = connexion(&self.pool)?;
        conn.query_row(
            &format!("SELECT {COLONNES} FROM entreprises WHERE id = ?1"),
            [id.to_string()],
            ligne_vers_entreprise,
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("entreprise {id}")),
            other => traduire_erreur(other, "entreprise"),
        })
    }

    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        search: &str,
        company_type: Option<&str>,
    ) -> AppResult<Page<Entreprise>> {
        let conn = connexion(&self.pool)?;
        let page = page.max(1);
        let page_size = page_size.max(1);
        let needle = format!("%{}%", search.trim().to_lowercase());
        let selected_type = company_type.unwrap_or_default().trim().to_lowercase();
        let filtre = "WHERE (?1 = '%%' OR lower(nom) LIKE ?1 OR lower(coalesce(secteur, '')) LIKE ?1 OR lower(coalesce(ville, '')) LIKE ?1) AND (?2 = '' OR lower(trim(coalesce(type, ''))) = ?2)";
        let total: u64 = conn
            .query_row(
                &format!("SELECT count(*) FROM entreprises {filtre}"),
                rusqlite::params![needle, selected_type],
                |row| row.get(0),
            )
            .map_err(|e| traduire_erreur(e, "entreprises"))?;
        let offset = page.saturating_sub(1).saturating_mul(page_size);
        let mut requete = conn
            .prepare(&format!(
                "SELECT {COLONNES} FROM entreprises {filtre} ORDER BY nom COLLATE NOCASE ASC LIMIT ?3 OFFSET ?4"
            ))
            .map_err(|e| traduire_erreur(e, "entreprises"))?;
        let lignes = requete
            .query_map(
                rusqlite::params![needle, selected_type, page_size, offset],
                ligne_vers_entreprise,
            )
            .map_err(|e| traduire_erreur(e, "entreprises"))?;
        let mut items = Vec::new();
        for ligne in lignes {
            items.push(ligne.map_err(|e| traduire_erreur(e, "entreprises"))?);
        }
        Ok(Page::new(items, total, page, page_size))
    }

    fn list_types(&self) -> AppResult<Vec<String>> {
        let conn = connexion(&self.pool)?;
        let mut query = conn
            .prepare(
                "SELECT trim(type) FROM entreprises WHERE trim(coalesce(type, '')) <> '' GROUP BY lower(trim(type)) ORDER BY lower(trim(type)) ASC",
            )
            .map_err(|e| traduire_erreur(e, "types d'entreprise"))?;
        let rows = query
            .query_map([], |row| row.get(0))
            .map_err(|e| traduire_erreur(e, "types d'entreprise"))?;
        rows.collect::<Result<Vec<String>, _>>()
            .map_err(|e| traduire_erreur(e, "types d'entreprise"))
    }

    fn create(&self, input: &NouvelleEntreprise) -> AppResult<Entreprise> {
        let conn = connexion(&self.pool)?;
        let id = uuid::Uuid::new_v4();
        let maintenant = maintenant_iso();
        conn.execute(
            "INSERT INTO entreprises (id, nom, secteur_id, secteur, type, site_web, ville, adresse, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
            rusqlite::params![
                id.to_string(),
                input.nom,
                input.secteur_id.map(|value| value.to_string()),
                input.secteur,
                input.type_,
                input.site_web,
                input.ville,
                input.adresse,
                input.notes,
                maintenant
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
                "UPDATE entreprises SET nom = ?2, secteur_id = ?3, secteur = ?4, type = ?5,
                    site_web = ?6, ville = ?7, adresse = ?8, notes = ?9, updated_at = ?10
                 WHERE id = ?1",
                rusqlite::params![
                    id.to_string(),
                    input.nom,
                    input.secteur_id.map(|value| value.to_string()),
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
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
