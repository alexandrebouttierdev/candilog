//! Dépôt `SQLite` de l'historique des relations.
//!
//! L'historique est lu à chaque ouverture de fiche dans les tables qui portent les faits
//! (candidatures, statuts, entretiens, relances) : seules les notes ont leur propre table.

use crate::core::database::helpers::{connection, now_iso, translate_error, uuid_column_opt};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::relations::domain::{
    HistoryEntry, HistoryKind, NewRelationNote, RelationHistoryRepository, RelationKind,
    RelationNote, RelationRef,
};
use uuid::Uuid;

/// Implémentation `SQLite` de l'historique des relations.
pub struct SqliteRelationHistoryRepository {
    pub(crate) pool: SqlitePool,
}

impl SqliteRelationHistoryRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Table de la fiche, colonne de note et condition « candidature liée à la fiche ».
const fn shape(kind: RelationKind) -> (&'static str, &'static str, &'static str) {
    match kind {
        RelationKind::Company => ("companies", "company_id", "a.company_id = ?1"),
        RelationKind::Contact => ("contacts", "contact_id", "a.contact_id = ?1"),
    }
}

fn kind_from_text(text: &str) -> AppResult<HistoryKind> {
    Ok(match text {
        "note" => HistoryKind::Note,
        "interview" => HistoryKind::Interview,
        "follow_up_done" => HistoryKind::FollowUpDone,
        "status_changed" => HistoryKind::StatusChanged,
        "application_sent" => HistoryKind::ApplicationSent,
        "added" => HistoryKind::Added,
        other => {
            return Err(AppError::Database(format!(
                "entrée d'historique inconnue : {other}"
            )))
        }
    })
}

fn ensure_exists(conn: &rusqlite::Connection, relation: RelationRef) -> AppResult<()> {
    let (table, _, _) = shape(relation.kind);
    let found: bool = conn
        .query_row(
            &format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id = ?1)"),
            [relation.id.to_string()],
            |row| row.get(0),
        )
        .map_err(|e| translate_error(e, "relation"))?;
    if found {
        Ok(())
    } else {
        Err(AppError::NotFound(format!("relation {}", relation.id)))
    }
}

impl RelationHistoryRepository for SqliteRelationHistoryRepository {
    fn history(&self, relation: RelationRef, limit: usize) -> AppResult<Vec<HistoryEntry>> {
        let conn = connection(&self.pool)?;
        ensure_exists(&conn, relation)?;
        let (table, note_column, linked) = shape(relation.kind);
        // Un entretien concerne un contact s'il y est nommé ou si sa candidature l'est.
        let interview_linked = match relation.kind {
            RelationKind::Company => "a.company_id = ?1",
            RelationKind::Contact => "(i.contact_id = ?1 OR a.contact_id = ?1)",
        };
        // Le premier statut d'une candidature est celui de sa création, déjà dit par
        // « candidature envoyée » : seuls les changements suivants entrent dans l'historique.
        let sql = format!(
            "SELECT kind, at, application_id, job_title, detail, note_id FROM (
                SELECT 'note' AS kind, n.noted_on AS at, NULL AS application_id,
                       NULL AS job_title, n.body AS detail, n.id AS note_id,
                       n.created_at AS written_at
                  FROM relation_notes n WHERE n.{note_column} = ?1
                UNION ALL
                SELECT 'application_sent', a.sent_date, a.id, a.job_title, NULL, NULL,
                       a.created_at
                  FROM applications a WHERE {linked}
                UNION ALL
                SELECT 'status_changed', h.changed_at, a.id, a.job_title, h.status, NULL,
                       h.changed_at
                  FROM status_history h JOIN applications a ON a.id = h.application_id
                 WHERE {linked}
                   AND h.changed_at > (SELECT min(first.changed_at) FROM status_history first
                                        WHERE first.application_id = h.application_id)
                UNION ALL
                SELECT 'interview', i.interview_date, a.id, a.job_title, i.type, NULL,
                       i.created_at
                  FROM interviews i JOIN applications a ON a.id = i.application_id
                 WHERE {interview_linked}
                UNION ALL
                SELECT 'follow_up_done', f.done_at, a.id, a.job_title, f.type, NULL, f.done_at
                  FROM follow_ups f JOIN applications a ON a.id = f.application_id
                 WHERE {linked} AND f.done_at IS NOT NULL
                UNION ALL
                SELECT 'added', r.created_at, NULL, NULL, NULL, NULL, r.created_at
                  FROM {table} r WHERE r.id = ?1
             )
             ORDER BY at DESC, written_at DESC
             LIMIT ?2"
        );
        let mut query = conn
            .prepare(&sql)
            .map_err(|e| translate_error(e, "historique"))?;
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let mut rows = query
            .query(rusqlite::params![relation.id.to_string(), limit])
            .map_err(|e| translate_error(e, "historique"))?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next().map_err(|e| translate_error(e, "historique"))? {
            let read_err = |e| translate_error(e, "historique");
            let kind: String = row.get(0).map_err(read_err)?;
            entries.push(HistoryEntry {
                kind: kind_from_text(&kind)?,
                at: row.get(1).map_err(read_err)?,
                application_id: uuid_column_opt(row, 2).map_err(read_err)?,
                job_title: row.get(3).map_err(read_err)?,
                detail: row.get(4).map_err(read_err)?,
                note_id: uuid_column_opt(row, 5).map_err(read_err)?,
            });
        }
        Ok(entries)
    }

    fn add_note(&self, input: &NewRelationNote) -> AppResult<RelationNote> {
        let conn = connection(&self.pool)?;
        ensure_exists(&conn, input.relation)?;
        let (_, note_column, _) = shape(input.relation.kind);
        let id = Uuid::new_v4();
        let created_at = now_iso();
        conn.execute(
            &format!(
                "INSERT INTO relation_notes (id, {note_column}, body, noted_on, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)"
            ),
            rusqlite::params![
                id.to_string(),
                input.relation.id.to_string(),
                input.body,
                input.noted_on,
                created_at,
            ],
        )
        .map_err(|e| translate_error(e, "note"))?;
        Ok(RelationNote {
            id,
            relation: input.relation,
            body: input.body.clone(),
            noted_on: input.noted_on.clone(),
            created_at,
        })
    }

    fn delete_note(&self, id: Uuid) -> AppResult<()> {
        let conn = connection(&self.pool)?;
        let deleted = conn
            .execute("DELETE FROM relation_notes WHERE id = ?1", [id.to_string()])
            .map_err(|e| translate_error(e, "note"))?;
        if deleted == 0 {
            return Err(AppError::NotFound(format!("note {id}")));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
