//! Persistance dans les tables historiques `resume_versions` et `cover_letters_motivation`.

use crate::core::database::helpers::{
    connection, like_contains, now_iso, translate_error, uuid_column, LIKE_ESCAPE,
};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::{clamp_page_size, Page};
use crate::features::documents::domain::{
    CoverLetter, CoverLetterRepository, NewCoverLetter, NewResume, ResumeRepository, ResumeSummary,
    ResumeVersion,
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

/// Score ATS d'une version, lu dans son JSON : éditeur v1 (`score.total`) ou ancienne
/// génération (`profile_score.total`). `json_valid` protège d'un contenu illisible, qui
/// n'a alors simplement pas de score.
const RESUME_SCORE: &str = "CASE WHEN json_valid(content) THEN CAST(round(coalesce( \
                              json_extract(content, '$.score.total'), \
                              json_extract(content, '$.profile_score.total'))) AS INTEGER) END";

/// Intitulé de l'offre ciblée par une version de l'éditeur v1.
const RESUME_TARGET: &str = "CASE WHEN json_valid(content) \
                               THEN nullif(json_extract(content, '$.job_offer.title'), '') END";

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

    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        search: &str,
        scored_only: bool,
    ) -> AppResult<Page<ResumeSummary>> {
        let conn = connection(&self.pool)?;
        let pattern = like_contains(search);
        let where_clause = format!(
            "search_key(name) LIKE ?1 {LIKE_ESCAPE}{}",
            if scored_only {
                format!(" AND {RESUME_SCORE} IS NOT NULL")
            } else {
                String::new()
            }
        );
        let total: u64 = conn
            .query_row(
                &format!("SELECT count(*) FROM resume_versions WHERE {where_clause}"),
                [&pattern],
                |row| row.get(0),
            )
            .map_err(|error| translate_error(error, "versions de CV"))?;
        let page_size = clamp_page_size(page_size);
        let offset = Page::<ResumeSummary>::offset(page, page_size);
        let mut query = conn
            .prepare(&format!(
                "SELECT id, name, created_at, {RESUME_SCORE}, {RESUME_TARGET} \
                 FROM resume_versions WHERE {where_clause} \
                 ORDER BY created_at DESC, rowid DESC LIMIT ?2 OFFSET ?3"
            ))
            .map_err(|error| translate_error(error, "versions de CV"))?;
        let rows = query
            .query_map(rusqlite::params![pattern, page_size, offset], |row| {
                Ok(ResumeSummary {
                    id: uuid_column(row, 0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    ats_score: row.get(3)?,
                    target_title: row.get(4)?,
                })
            })
            .map_err(|error| translate_error(error, "versions de CV"))?;
        let items = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| translate_error(error, "versions de CV"))?;
        Ok(Page::new(items, total, page, page_size))
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
            .execute(
                "DELETE FROM resume_versions WHERE id = ?1",
                [id.to_string()],
            )
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
const COVER_LETTER_COLUMNS: &str =
    "id, name, company, job_title, recipient, recipient_address, job_reference, tone, length, content, created_at";
fn cover_letter_row(row: &rusqlite::Row) -> rusqlite::Result<CoverLetter> {
    Ok(CoverLetter {
        id: uuid_column(row, 0)?,
        name: row.get(1)?,
        company: row.get(2)?,
        job_title: row.get(3)?,
        recipient: row.get(4)?,
        recipient_address: row.get(5)?,
        job_reference: row.get(6)?,
        tone: row.get(7)?,
        length: row.get(8)?,
        content: row.get(9)?,
        created_at: row.get(10)?,
    })
}

impl CoverLetterRepository for SqliteCoverLetterRepository {
    fn save(&self, input: &NewCoverLetter) -> AppResult<CoverLetter> {
        let conn = connection(&self.pool)?;
        let id = Uuid::new_v4();
        let created_at = now_iso();
        conn.execute(
            "INSERT INTO cover_letters (id, name, company, job_title, recipient, recipient_address, job_reference, tone, length, content, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                id.to_string(),
                input.name,
                input.company,
                input.job_title,
                input.recipient,
                input.recipient_address,
                input.job_reference,
                input.tone,
                input.length,
                input.content,
                created_at
            ],
        )
        .map_err(|e| translate_error(e, "lettre de motivation"))?;
        Ok(CoverLetter {
            id,
            name: input.name.clone(),
            company: input.company.clone(),
            job_title: input.job_title.clone(),
            recipient: input.recipient.clone(),
            recipient_address: input.recipient_address.clone(),
            job_reference: input.job_reference.clone(),
            tone: input.tone.clone(),
            length: input.length.clone(),
            content: input.content.clone(),
            created_at,
        })
    }

    fn list_page(&self, page: u64, page_size: u64, search: &str) -> AppResult<Page<CoverLetter>> {
        let conn = connection(&self.pool)?;
        let pattern = like_contains(search);
        let where_clause = format!(
            "(search_key(name) LIKE ?1 {LIKE_ESCAPE} \
             OR search_key(coalesce(company, '')) LIKE ?1 {LIKE_ESCAPE} \
             OR search_key(coalesce(job_title, '')) LIKE ?1 {LIKE_ESCAPE})"
        );
        let total: u64 = conn
            .query_row(
                &format!("SELECT count(*) FROM cover_letters WHERE {where_clause}"),
                [&pattern],
                |row| row.get(0),
            )
            .map_err(|error| translate_error(error, "lettres de motivation"))?;
        let page_size = clamp_page_size(page_size);
        let offset = Page::<CoverLetter>::offset(page, page_size);
        let mut query = conn
            .prepare(&format!(
                "SELECT {COVER_LETTER_COLUMNS} FROM cover_letters WHERE {where_clause} \
                 ORDER BY created_at DESC, rowid DESC LIMIT ?2 OFFSET ?3"
            ))
            .map_err(|error| translate_error(error, "lettres de motivation"))?;
        let rows = query
            .query_map(
                rusqlite::params![pattern, page_size, offset],
                cover_letter_row,
            )
            .map_err(|error| translate_error(error, "lettres de motivation"))?;
        let items = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| translate_error(error, "lettres de motivation"))?;
        Ok(Page::new(items, total, page, page_size))
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
            .execute("DELETE FROM cover_letters WHERE id = ?1", [id.to_string()])
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
