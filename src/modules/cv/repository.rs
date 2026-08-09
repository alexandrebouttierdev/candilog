//! Accès aux versions de CV (base locale `SQLite`).

use crate::modules::cv::model::{CvVersion, CvVersionSummary};
use crate::shared::db::SqlitePool;
use crate::shared::error::{AppError, AppResult};
use crate::shared::sqlite::{connexion, maintenant_iso, traduire_erreur, uuid_colonne};
use serde_json::Value;
use uuid::Uuid;

/// Contrat d'accès aux versions de CV.
pub trait CvVersionRepository: Send + Sync {
    /// Crée une version de CV.
    ///
    /// # Errors
    /// `AppError::Serialization` si le contenu ne peut pas être sérialisé ; sinon
    /// `AppError::Database`.
    fn create(&self, name: &str, content: &Value) -> AppResult<CvVersion>;
    /// Liste les résumés des versions (les plus récentes d'abord), sans le contenu.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<CvVersionSummary>>;
    /// Récupère une version complète par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si absente ; `AppError::Serialization` si le contenu stocké est
    /// invalide ; sinon `AppError::Database`.
    fn get(&self, id: Uuid) -> AppResult<CvVersion>;
    /// Supprime une version par identifiant.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la suppression échoue.
    fn delete(&self, id: Uuid) -> AppResult<()>;
}

/// Implémentation `SQLite` du dépôt de versions de CV.
pub struct SqliteCvVersionRepository {
    pool: SqlitePool,
}

impl SqliteCvVersionRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl CvVersionRepository for SqliteCvVersionRepository {
    fn create(&self, name: &str, content: &Value) -> AppResult<CvVersion> {
        let conn = connexion(&self.pool)?;
        let id = Uuid::new_v4();
        let maintenant = maintenant_iso();
        let contenu_texte =
            serde_json::to_string(content).map_err(|e| AppError::Serialization(e.to_string()))?;
        conn.execute(
            "INSERT INTO cv_versions (id, name, content, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id.to_string(), name, contenu_texte, maintenant],
        )
        .map_err(|e| traduire_erreur(e, "version de CV"))?;
        Ok(CvVersion {
            id,
            name: name.to_owned(),
            content: content.clone(),
            created_at: maintenant,
        })
    }

    fn list(&self) -> AppResult<Vec<CvVersionSummary>> {
        let conn = connexion(&self.pool)?;
        let mut requete = conn
            .prepare(
                "SELECT id, name, created_at FROM cv_versions ORDER BY created_at DESC, rowid DESC",
            )
            .map_err(|e| traduire_erreur(e, "versions de CV"))?;
        let mut lignes = requete
            .query([])
            .map_err(|e| traduire_erreur(e, "versions de CV"))?;
        let mut resumes = Vec::new();
        while let Some(row) = lignes
            .next()
            .map_err(|e| traduire_erreur(e, "versions de CV"))?
        {
            resumes.push(CvVersionSummary {
                id: uuid_colonne(row, 0).map_err(|e| traduire_erreur(e, "version de CV"))?,
                name: row
                    .get(1)
                    .map_err(|e| traduire_erreur(e, "version de CV"))?,
                created_at: row
                    .get(2)
                    .map_err(|e| traduire_erreur(e, "version de CV"))?,
            });
        }
        Ok(resumes)
    }

    fn get(&self, id: Uuid) -> AppResult<CvVersion> {
        let conn = connexion(&self.pool)?;
        let (name, contenu_texte, created_at): (String, String, String) = conn
            .query_row(
                "SELECT name, content, created_at FROM cv_versions WHERE id = ?1",
                [id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| traduire_erreur(e, &format!("version de CV {id}")))?;
        let content = serde_json::from_str(&contenu_texte)
            .map_err(|e| AppError::Serialization(e.to_string()))?;
        Ok(CvVersion {
            id,
            name,
            content,
            created_at,
        })
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = connexion(&self.pool)?;
        conn.execute("DELETE FROM cv_versions WHERE id = ?1", [id.to_string()])
            .map_err(|e| traduire_erreur(e, "version de CV"))?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/repository/mod.rs"]
mod tests;
