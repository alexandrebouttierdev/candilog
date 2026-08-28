//! Persistance dans les tables historiques `resume_versions` et `cover_letters_motivation`.

use crate::core::database::helpers::{connection, now_iso, translate_error, uuid_column};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::documents::domain::{
    ResumeRepository, ResumeSummary, ResumeVersion, CoverLetter, CoverLetterRepository, NewResume, NewCoverLetter,
};
use uuid::Uuid;

pub struct SqliteResumeRepository {
    pool: SqlitePool,
}
impl SqliteResumeRepository {
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl ResumeRepository for SqliteResumeRepository {
    fn save(&self, input: &NewResume) -> AppResult<ResumeVersion> {
        let conn = connection(&self.pool)?;
        let id = Uuid::new_v4();
        let created_at = now_iso();
        let content = serde_json::to_string(&input.content)
            .map_err(|e| AppError::Serialization(e.to_string()))?;
        conn.execute(
            "INSERT INTO resume_versions (id, name, content, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![id.to_string(), input.name, content, created_at],
        )
        .map_err(|e| translate_error(e, "version de CV"))?;
        Ok(ResumeVersion {
            id,
            name: input.name.clone(),
            content: input.content.clone(),
            created_at,
        })
    }

    fn list(&self) -> AppResult<Vec<ResumeSummary>> {
        let conn = connection(&self.pool)?;
        let mut query = conn
            .prepare(
                "SELECT id, name, created_at FROM resume_versions ORDER BY created_at DESC, rowid DESC",
            )
            .map_err(|e| translate_error(e, "versions de CV"))?;
        let rows = query
            .query_map([], |row| {
                Ok(ResumeSummary {
                    id: uuid_column(row, 0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })
            .map_err(|e| translate_error(e, "versions de CV"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| translate_error(e, "versions de CV"))
    }

    fn get(&self, id: Uuid) -> AppResult<ResumeVersion> {
        let conn = connection(&self.pool)?;
        let (name, raw, created_at): (String, String, String) = conn
            .query_row(
                "SELECT name, content, created_at FROM resume_versions WHERE id = ?1",
                [id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| translate_error(e, &format!("version de CV {id}")))?;
        let content =
            serde_json::from_str(&raw).map_err(|e| AppError::Serialization(e.to_string()))?;
        Ok(ResumeVersion {
            id,
            name,
            content,
            created_at,
        })
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = connection(&self.pool)?;
        let count = conn
            .execute("DELETE FROM resume_versions WHERE id = ?1", [id.to_string()])
            .map_err(|e| translate_error(e, "version de CV"))?;
        if count == 0 {
            return Err(AppError::NotFound(format!("version de CV {id}")));
        }
        Ok(())
    }
}

pub struct SqliteCoverLetterRepository {
    pool: SqlitePool,
}
impl SqliteCoverLetterRepository {
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
const COVER_LETTER_COLUMNS: &str = "id, name, company, job_title, tone, length, content, created_at";
fn cover_letter_row(row: &rusqlite::Row) -> rusqlite::Result<CoverLetter> {
    Ok(CoverLetter {
        id: uuid_column(row, 0)?,
        name: row.get(1)?,
        company: row.get(2)?,
        job_title: row.get(3)?,
        tone: row.get(4)?,
        length: row.get(5)?,
        content: row.get(6)?,
        created_at: row.get(7)?,
    })
}

impl CoverLetterRepository for SqliteCoverLetterRepository {
    fn save(&self, input: &NewCoverLetter) -> AppResult<CoverLetter> {
        let conn = connection(&self.pool)?;
        let id = Uuid::new_v4();
        let created_at = now_iso();
        conn.execute("INSERT INTO cover_letters (id, name, company, job_title, tone, length, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", rusqlite::params![id.to_string(), input.name, input.company, input.job_title, input.tone, input.length, input.content, created_at]).map_err(|e| translate_error(e, "lettre de motivation"))?;
        Ok(CoverLetter {
            id,
            name: input.name.clone(),
            company: input.company.clone(),
            job_title: input.job_title.clone(),
            tone: input.tone.clone(),
            length: input.length.clone(),
            content: input.content.clone(),
            created_at,
        })
    }

    fn list(&self) -> AppResult<Vec<CoverLetter>> {
        let conn = connection(&self.pool)?;
        let mut query = conn.prepare(&format!("SELECT {COVER_LETTER_COLUMNS} FROM cover_letters ORDER BY created_at DESC, rowid DESC")).map_err(|e| translate_error(e, "lettres de motivation"))?;
        let rows = query
            .query_map([], cover_letter_row)
            .map_err(|e| translate_error(e, "lettres de motivation"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| translate_error(e, "lettres de motivation"))
    }

    fn get(&self, id: Uuid) -> AppResult<CoverLetter> {
        connection(&self.pool)?
            .query_row(
                &format!("SELECT {COVER_LETTER_COLUMNS} FROM cover_letters WHERE id = ?1"),
                [id.to_string()],
                cover_letter_row,
            )
            .map_err(|e| translate_error(e, &format!("lettre de motivation {id}")))
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let count = connection(&self.pool)?
            .execute(
                "DELETE FROM cover_letters WHERE id = ?1",
                [id.to_string()],
            )
            .map_err(|e| translate_error(e, "lettre de motivation"))?;
        if count == 0 {
            return Err(AppError::NotFound(format!("lettre de motivation {id}")));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
