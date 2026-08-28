//! Persistance dans les tables historiques `cv_versions` et `lettres_motivation`.

use crate::core::database::helpers::{connexion, maintenant_iso, traduire_erreur, uuid_colonne};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::documents::domain::{
    CvRepository, CvResume, CvVersion, Lettre, LettreRepository, NouveauCv, NouvelleLettre,
};
use uuid::Uuid;

pub struct SqliteCvRepository {
    pool: SqlitePool,
}
impl SqliteCvRepository {
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl CvRepository for SqliteCvRepository {
    fn enregistrer(&self, input: &NouveauCv) -> AppResult<CvVersion> {
        let conn = connexion(&self.pool)?;
        let id = Uuid::new_v4();
        let created_at = maintenant_iso();
        let contenu = serde_json::to_string(&input.contenu)
            .map_err(|e| AppError::Serialization(e.to_string()))?;
        conn.execute(
            "INSERT INTO cv_versions (id, name, content, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id.to_string(), input.nom, contenu, created_at],
        )
        .map_err(|e| traduire_erreur(e, "version de CV"))?;
        Ok(CvVersion {
            id,
            nom: input.nom.clone(),
            contenu: input.contenu.clone(),
            created_at,
        })
    }

    fn lister(&self) -> AppResult<Vec<CvResume>> {
        let conn = connexion(&self.pool)?;
        let mut query = conn
            .prepare(
                "SELECT id, name, created_at FROM cv_versions ORDER BY created_at DESC, rowid DESC",
            )
            .map_err(|e| traduire_erreur(e, "versions de CV"))?;
        let rows = query
            .query_map([], |row| {
                Ok(CvResume {
                    id: uuid_colonne(row, 0)?,
                    nom: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })
            .map_err(|e| traduire_erreur(e, "versions de CV"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| traduire_erreur(e, "versions de CV"))
    }

    fn obtenir(&self, id: Uuid) -> AppResult<CvVersion> {
        let conn = connexion(&self.pool)?;
        let (nom, brut, created_at): (String, String, String) = conn
            .query_row(
                "SELECT name, content, created_at FROM cv_versions WHERE id = ?1",
                [id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| traduire_erreur(e, &format!("version de CV {id}")))?;
        let contenu =
            serde_json::from_str(&brut).map_err(|e| AppError::Serialization(e.to_string()))?;
        Ok(CvVersion {
            id,
            nom,
            contenu,
            created_at,
        })
    }

    fn supprimer(&self, id: Uuid) -> AppResult<()> {
        let conn = connexion(&self.pool)?;
        let count = conn
            .execute("DELETE FROM cv_versions WHERE id = ?1", [id.to_string()])
            .map_err(|e| traduire_erreur(e, "version de CV"))?;
        if count == 0 {
            return Err(AppError::NotFound(format!("version de CV {id}")));
        }
        Ok(())
    }
}

pub struct SqliteLettreRepository {
    pool: SqlitePool,
}
impl SqliteLettreRepository {
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
const LETTRE_COLONNES: &str = "id, name, company, job_title, tone, length, content, created_at";
fn lettre_row(row: &rusqlite::Row) -> rusqlite::Result<Lettre> {
    Ok(Lettre {
        id: uuid_colonne(row, 0)?,
        nom: row.get(1)?,
        entreprise: row.get(2)?,
        poste: row.get(3)?,
        ton: row.get(4)?,
        longueur: row.get(5)?,
        contenu: row.get(6)?,
        created_at: row.get(7)?,
    })
}

impl LettreRepository for SqliteLettreRepository {
    fn enregistrer(&self, input: &NouvelleLettre) -> AppResult<Lettre> {
        let conn = connexion(&self.pool)?;
        let id = Uuid::new_v4();
        let created_at = maintenant_iso();
        conn.execute("INSERT INTO lettres_motivation (id, name, company, job_title, tone, length, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", rusqlite::params![id.to_string(), input.nom, input.entreprise, input.poste, input.ton, input.longueur, input.contenu, created_at]).map_err(|e| traduire_erreur(e, "lettre de motivation"))?;
        Ok(Lettre {
            id,
            nom: input.nom.clone(),
            entreprise: input.entreprise.clone(),
            poste: input.poste.clone(),
            ton: input.ton.clone(),
            longueur: input.longueur.clone(),
            contenu: input.contenu.clone(),
            created_at,
        })
    }

    fn lister(&self) -> AppResult<Vec<Lettre>> {
        let conn = connexion(&self.pool)?;
        let mut query = conn.prepare(&format!("SELECT {LETTRE_COLONNES} FROM lettres_motivation ORDER BY created_at DESC, rowid DESC")).map_err(|e| traduire_erreur(e, "lettres de motivation"))?;
        let rows = query
            .query_map([], lettre_row)
            .map_err(|e| traduire_erreur(e, "lettres de motivation"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| traduire_erreur(e, "lettres de motivation"))
    }

    fn obtenir(&self, id: Uuid) -> AppResult<Lettre> {
        connexion(&self.pool)?
            .query_row(
                &format!("SELECT {LETTRE_COLONNES} FROM lettres_motivation WHERE id = ?1"),
                [id.to_string()],
                lettre_row,
            )
            .map_err(|e| traduire_erreur(e, &format!("lettre de motivation {id}")))
    }

    fn supprimer(&self, id: Uuid) -> AppResult<()> {
        let count = connexion(&self.pool)?
            .execute(
                "DELETE FROM lettres_motivation WHERE id = ?1",
                [id.to_string()],
            )
            .map_err(|e| traduire_erreur(e, "lettre de motivation"))?;
        if count == 0 {
            return Err(AppError::NotFound(format!("lettre de motivation {id}")));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
