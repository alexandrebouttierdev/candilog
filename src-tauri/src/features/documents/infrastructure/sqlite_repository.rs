//! Persistance dans les tables historiques `resume_versions` et `cover_letters`, versionnées
//! depuis la migration 8 : une ligne par version, reliées par `document_id`.

use crate::core::database::helpers::{
    connection, like_contains, now_iso, translate_error, uuid_column, LIKE_ESCAPE,
};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::{clamp_page_size, Page};
use crate::features::documents::domain::{
    CoverLetter, CoverLetterRepository, DocumentVersion, NewCoverLetter, NewResume,
    ResumeRepository, ResumeSummary, ResumeVersion,
};
use uuid::Uuid;

/// Bibliothèque versionnée : chaque ligne est une version, `document_id` relie celles d'un
/// même document et `is_current` désigne celle que la bibliothèque affiche (migration 8).
#[derive(Clone, Copy)]
enum Library {
    Resumes,
    Letters,
}

impl Library {
    const fn table(self) -> &'static str {
        match self {
            Self::Resumes => "resume_versions",
            Self::Letters => "cover_letters",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Resumes => "CV",
            Self::Letters => "lettre de motivation",
        }
    }
}

/// Document auquel appartient la version `id`.
fn document_of(conn: &rusqlite::Connection, library: Library, id: Uuid) -> AppResult<String> {
    conn.query_row(
        &format!(
            "SELECT coalesce(document_id, id) FROM {} WHERE id = ?1",
            library.table()
        ),
        [id.to_string()],
        |row| row.get(0),
    )
    .map_err(|e| translate_error(e, &format!("{} {id}", library.label())))
}

/// Document et numéro de la version à insérer : v1 d'un nouveau document, ou la suivante
/// de celui que l'on révise, dont les autres versions cessent alors d'être courantes.
fn next_version(
    conn: &rusqlite::Connection,
    library: Library,
    id: Uuid,
    revises: Option<Uuid>,
) -> AppResult<(String, u32)> {
    let Some(revised) = revises else {
        return Ok((id.to_string(), 1));
    };
    let table = library.table();
    let document = document_of(conn, library, revised)?;
    let next: u32 = conn
        .query_row(
            &format!("SELECT max(version_number) + 1 FROM {table} WHERE document_id = ?1"),
            [&document],
            |row| row.get(0),
        )
        .map_err(|e| translate_error(e, library.label()))?;
    conn.execute(
        &format!("UPDATE {table} SET is_current = 0 WHERE document_id = ?1"),
        [&document],
    )
    .map_err(|e| translate_error(e, library.label()))?;
    Ok((document, next))
}

fn list_versions(
    conn: &rusqlite::Connection,
    library: Library,
    id: Uuid,
) -> AppResult<Vec<DocumentVersion>> {
    let document = document_of(conn, library, id)?;
    let mut query = conn
        .prepare(&format!(
            "SELECT id, version_number, version_note, created_at, is_current FROM {} \
             WHERE document_id = ?1 ORDER BY version_number DESC",
            library.table()
        ))
        .map_err(|e| translate_error(e, library.label()))?;
    let rows = query
        .query_map([&document], |row| {
            Ok(DocumentVersion {
                id: uuid_column(row, 0)?,
                version_number: row.get(1)?,
                note: row.get(2)?,
                created_at: row.get(3)?,
                is_current: row.get(4)?,
            })
        })
        .map_err(|e| translate_error(e, library.label()))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| translate_error(e, library.label()))
}

fn restore_version(pool: &SqlitePool, library: Library, id: Uuid) -> AppResult<()> {
    let mut conn = connection(pool)?;
    let transaction = conn
        .transaction()
        .map_err(|e| translate_error(e, library.label()))?;
    let document = document_of(&transaction, library, id)?;
    let table = library.table();
    transaction
        .execute(
            &format!("UPDATE {table} SET is_current = 0 WHERE document_id = ?1"),
            [&document],
        )
        .map_err(|e| translate_error(e, library.label()))?;
    transaction
        .execute(
            &format!("UPDATE {table} SET is_current = 1 WHERE id = ?1"),
            [id.to_string()],
        )
        .map_err(|e| translate_error(e, library.label()))?;
    transaction
        .commit()
        .map_err(|e| translate_error(e, library.label()))
}

fn delete_document(pool: &SqlitePool, library: Library, id: Uuid) -> AppResult<()> {
    let conn = connection(pool)?;
    let document = document_of(&conn, library, id)?;
    conn.execute(
        &format!("DELETE FROM {} WHERE document_id = ?1", library.table()),
        [&document],
    )
    .map_err(|e| translate_error(e, library.label()))?;
    Ok(())
}

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
        let mut conn = connection(&self.pool)?;
        let id = Uuid::new_v4();
        let created_at = now_iso();
        let content = serde_json::to_string(&input.content)
            .map_err(|e| AppError::Serialization(e.to_string()))?;
        let transaction = conn
            .transaction()
            .map_err(|e| translate_error(e, "version de CV"))?;
        let (document, number) = next_version(&transaction, Library::Resumes, id, input.revises)?;
        transaction
            .execute(
                "INSERT INTO resume_versions \
                 (id, name, content, created_at, document_id, version_number, is_current, version_note) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7)",
                rusqlite::params![
                    id.to_string(),
                    input.name,
                    content,
                    created_at,
                    document,
                    number,
                    input.version_note
                ],
            )
            .map_err(|e| translate_error(e, "version de CV"))?;
        transaction
            .commit()
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
            "is_current = 1 AND search_key(name) LIKE ?1 {LIKE_ESCAPE}{}",
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

    fn versions(&self, id: Uuid) -> AppResult<Vec<DocumentVersion>> {
        let conn = connection(&self.pool)?;
        list_versions(&conn, Library::Resumes, id)
    }

    fn restore(&self, id: Uuid) -> AppResult<()> {
        restore_version(&self.pool, Library::Resumes, id)
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        delete_document(&self.pool, Library::Resumes, id)
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
        let mut conn = connection(&self.pool)?;
        let id = Uuid::new_v4();
        let created_at = now_iso();
        let transaction = conn
            .transaction()
            .map_err(|e| translate_error(e, "lettre de motivation"))?;
        let (document, number) = next_version(&transaction, Library::Letters, id, input.revises)?;
        transaction
            .execute(
                "INSERT INTO cover_letters (id, name, company, job_title, recipient, recipient_address, job_reference, tone, length, content, created_at, \
                 document_id, version_number, is_current, version_note) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 1, ?14)",
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
                    created_at,
                    document,
                    number,
                    input.version_note
                ],
            )
            .map_err(|e| translate_error(e, "lettre de motivation"))?;
        transaction
            .commit()
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
            "is_current = 1 AND (search_key(name) LIKE ?1 {LIKE_ESCAPE} \
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

    fn versions(&self, id: Uuid) -> AppResult<Vec<DocumentVersion>> {
        let conn = connection(&self.pool)?;
        list_versions(&conn, Library::Letters, id)
    }

    fn restore(&self, id: Uuid) -> AppResult<()> {
        restore_version(&self.pool, Library::Letters, id)
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        delete_document(&self.pool, Library::Letters, id)
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
