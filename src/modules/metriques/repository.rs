//! Persistance locale des métriques (`SQLite`) : télémétrie `LLM` et historique `ATS`.

use crate::modules::metriques::model::{
    AppelLlm, OperationLlm, OrigineScore, Page, ResumeScoresAts, ScoreAts,
};
use crate::shared::db::SqlitePool;
use crate::shared::error::{AppError, AppResult};

/// Contrat de persistance des métriques locales.
pub trait MetriquesRepository: Send + Sync {
    /// Enregistre un appel `LLM`.
    ///
    /// # Errors
    /// `AppError::Database` si l'écriture échoue.
    fn enregistrer_appel(&self, appel: &AppelLlm) -> AppResult<()>;

    /// Enregistre un score `ATS`.
    ///
    /// # Errors
    /// `AppError::Database` si l'écriture échoue.
    fn enregistrer_score(&self, score: &ScoreAts) -> AppResult<()>;

    /// Liste les appels `LLM` (plus récents d'abord).
    ///
    /// # Errors
    /// `AppError::Database` si la lecture échoue ; `AppError::Serialization` si une
    /// opération stockée est inconnue.
    fn lister_appels(&self) -> AppResult<Vec<AppelLlm>>;

    /// Liste les scores `ATS` (plus récents d'abord).
    ///
    /// # Errors
    /// `AppError::Database` si la lecture échoue ; `AppError::Serialization` si une
    /// origine stockée est inconnue.
    fn lister_scores(&self) -> AppResult<Vec<ScoreAts>>;

    /// Liste une page bornée des appels `LLM`.
    ///
    /// # Errors
    /// Retourne une erreur de base ou de conversion si la page ne peut pas être lue.
    fn lister_appels_page(&self, page: u64, page_size: u64) -> AppResult<Page<AppelLlm>>;

    /// Liste une page bornée des scores `ATS`.
    ///
    /// # Errors
    /// Retourne une erreur de base ou de conversion si la page ne peut pas être lue.
    fn lister_scores_page(&self, page: u64, page_size: u64) -> AppResult<Page<ScoreAts>>;

    /// Calcule les agrégats globaux des scores sans charger l'historique en mémoire.
    ///
    /// # Errors
    /// Retourne une erreur de base si les agrégats ne peuvent pas être calculés.
    fn resumer_scores(&self) -> AppResult<ResumeScoresAts>;

    /// Vide le journal des appels `LLM`.
    ///
    /// # Errors
    /// `AppError::Database` si la requête échoue.
    fn reset_appels(&self) -> AppResult<()>;

    /// Vide le journal des scores `ATS`.
    ///
    /// # Errors
    /// `AppError::Database` si la requête échoue.
    fn reset_scores(&self) -> AppResult<()>;
}

/// Implémentation `SQLite` du dépôt de métriques.
pub struct SqliteMetriquesRepository {
    pool: SqlitePool,
}

impl SqliteMetriquesRepository {
    /// Construit le dépôt à partir du pool `SQLite` partagé.
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Ligne brute d'un appel `LLM` (`operation` encore en texte).
type AppelRow = (String, String, String, i64, i64, String);

/// Reconstruit un `AppelLlm` hors closure `rusqlite`.
fn ligne_vers_appel(row: AppelRow) -> AppResult<AppelLlm> {
    let (operation, provider, modele, latence_ms, succes, cree_le) = row;
    Ok(AppelLlm {
        operation: OperationLlm::depuis_str(&operation).ok_or_else(|| {
            AppError::Serialization(format!("opération LLM inconnue : {operation}"))
        })?,
        provider,
        modele,
        latence_ms: u64::try_from(latence_ms).unwrap_or_default(),
        succes: succes != 0,
        cree_le,
    })
}

/// Ligne brute d'un score `ATS` (`origine` encore en texte).
type ScoreRow = (i64, String, String);

/// Reconstruit un `ScoreAts` hors closure `rusqlite`.
fn ligne_vers_score(row: ScoreRow) -> AppResult<ScoreAts> {
    let (score, origine, cree_le) = row;
    Ok(ScoreAts {
        score: u8::try_from(score).unwrap_or_default(),
        origine: OrigineScore::depuis_str(&origine).ok_or_else(|| {
            AppError::Serialization(format!("origine de score inconnue : {origine}"))
        })?,
        cree_le,
    })
}

impl MetriquesRepository for SqliteMetriquesRepository {
    #[allow(clippy::unnecessary_lazy_evaluations)]
    fn enregistrer_appel(&self, appel: &AppelLlm) -> AppResult<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute(
            "INSERT INTO llm_appels (operation, provider, modele, latence_ms, succes, cree_le)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                appel.operation.as_str(),
                appel.provider,
                appel.modele,
                i64::try_from(appel.latence_ms).unwrap_or_else(|_| i64::MAX),
                i64::from(appel.succes),
                appel.cree_le,
            ],
        )?;
        Ok(())
    }

    fn enregistrer_score(&self, score: &ScoreAts) -> AppResult<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute(
            "INSERT INTO scores_ats (score, origine, cree_le) VALUES (?1, ?2, ?3)",
            rusqlite::params![
                i64::from(score.score),
                score.origine.as_str(),
                score.cree_le
            ],
        )?;
        Ok(())
    }

    fn lister_appels(&self) -> AppResult<Vec<AppelLlm>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT operation, provider, modele, latence_ms, succes, cree_le
             FROM llm_appels ORDER BY cree_le DESC, id DESC",
        )?;
        let rows: Vec<AppelRow> = stmt
            .query_map([], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows.into_iter().map(ligne_vers_appel).collect()
    }

    fn lister_scores(&self) -> AppResult<Vec<ScoreAts>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT score, origine, cree_le FROM scores_ats ORDER BY cree_le DESC, id DESC",
        )?;
        let rows: Vec<ScoreRow> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        rows.into_iter().map(ligne_vers_score).collect()
    }

    fn lister_appels_page(&self, page: u64, page_size: u64) -> AppResult<Page<AppelLlm>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let total_i64: i64 =
            conn.query_row("SELECT count(*) FROM llm_appels", [], |row| row.get(0))?;
        let total = u64::try_from(total_i64).unwrap_or_default();
        let page = page.min(total.div_ceil(page_size).max(1));
        let offset = page.saturating_sub(1).saturating_mul(page_size);
        let limit_sql = i64::try_from(page_size)
            .map_err(|_| AppError::Validation("taille de page invalide".into()))?;
        let offset_sql = i64::try_from(offset)
            .map_err(|_| AppError::Validation("numéro de page invalide".into()))?;
        let mut stmt = conn.prepare(
            "SELECT operation, provider, modele, latence_ms, succes, cree_le
             FROM llm_appels ORDER BY cree_le DESC, id DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows: Vec<AppelRow> = stmt
            .query_map(rusqlite::params![limit_sql, offset_sql], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let items = rows
            .into_iter()
            .map(ligne_vers_appel)
            .collect::<AppResult<Vec<_>>>()?;
        Ok(Page::new(items, total, page, page_size))
    }

    fn lister_scores_page(&self, page: u64, page_size: u64) -> AppResult<Page<ScoreAts>> {
        let conn = self
            .pool
            .get()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let total_i64: i64 =
            conn.query_row("SELECT count(*) FROM scores_ats", [], |row| row.get(0))?;
        let total = u64::try_from(total_i64).unwrap_or_default();
        let page = page.min(total.div_ceil(page_size).max(1));
        let offset = page.saturating_sub(1).saturating_mul(page_size);
        let limit_sql = i64::try_from(page_size)
            .map_err(|_| AppError::Validation("taille de page invalide".into()))?;
        let offset_sql = i64::try_from(offset)
            .map_err(|_| AppError::Validation("numéro de page invalide".into()))?;
        let mut stmt = conn.prepare(
            "SELECT score, origine, cree_le FROM scores_ats
             ORDER BY cree_le DESC, id DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows: Vec<ScoreRow> = stmt
            .query_map(rusqlite::params![limit_sql, offset_sql], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let items = rows
            .into_iter()
            .map(ligne_vers_score)
            .collect::<AppResult<Vec<_>>>()?;
        Ok(Page::new(items, total, page, page_size))
    }

    fn resumer_scores(&self) -> AppResult<ResumeScoresAts> {
        let conn = self
            .pool
            .get()
            .map_err(|e| AppError::Database(e.to_string()))?;
        let valeurs: (i64, i64, i64, i64, i64, i64, i64, i64, i64, i64) = conn.query_row(
            "SELECT
                count(*),
                CAST(ROUND(COALESCE(AVG(score), 0)) AS INTEGER),
                COALESCE(SUM(CASE WHEN score BETWEEN 0 AND 49 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN score BETWEEN 50 AND 69 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN score BETWEEN 70 AND 84 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN score BETWEEN 85 AND 100 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN origine = 'genere' THEN 1 ELSE 0 END), 0),
                CAST(ROUND(COALESCE(AVG(CASE WHEN origine = 'genere' THEN score END), 0)) AS INTEGER),
                COALESCE(SUM(CASE WHEN origine = 'importe' THEN 1 ELSE 0 END), 0),
                CAST(ROUND(COALESCE(AVG(CASE WHEN origine = 'importe' THEN score END), 0)) AS INTEGER)
             FROM scores_ats",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?)),
        )?;
        Ok(ResumeScoresAts {
            nombre: u64::try_from(valeurs.0).unwrap_or_default(),
            moyenne: u8::try_from(valeurs.1).unwrap_or_default(),
            faibles: u64::try_from(valeurs.2).unwrap_or_default(),
            partiels: u64::try_from(valeurs.3).unwrap_or_default(),
            bons: u64::try_from(valeurs.4).unwrap_or_default(),
            excellents: u64::try_from(valeurs.5).unwrap_or_default(),
            generes_nombre: u64::try_from(valeurs.6).unwrap_or_default(),
            generes_moyenne: u8::try_from(valeurs.7).unwrap_or_default(),
            importes_nombre: u64::try_from(valeurs.8).unwrap_or_default(),
            importes_moyenne: u8::try_from(valeurs.9).unwrap_or_default(),
        })
    }

    fn reset_appels(&self) -> AppResult<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute("DELETE FROM llm_appels", [])?;
        Ok(())
    }

    fn reset_scores(&self) -> AppResult<()> {
        let conn = self
            .pool
            .get()
            .map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute("DELETE FROM scores_ats", [])?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/repository/mod.rs"]
mod tests;
