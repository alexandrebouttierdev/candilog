//! Dépôt `SQLite` des relances.

use crate::core::database::helpers::{
    connection, now_iso, translate_constraint, translate_error, uuid_column,
};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::followups::domain::{FollowUp, FollowUpRepository, NewFollowUp};
use uuid::Uuid;

/// Implémentation `SQLite` du dépôt de relances.
pub struct SqliteFollowUpRepository {
    pool: SqlitePool,
}

impl SqliteFollowUpRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Columns lues par [`row_vers_follow_up`], dans l'ordre.
const COLUMNS: &str = "r.id, r.application_id, c.job_title, e.name, r.follow_up_date, r.type, \
                        r.notes, r.created_at";

/// Source des colonnes : jointures pour afficher le poste et l'entreprise au calendrier.
const FROM_SQL: &str = "FROM follow_ups r \
                      LEFT JOIN applications c ON c.id = r.application_id \
                      LEFT JOIN companies e ON e.id = c.company_id";

/// Convertit une ligne `SQLite` en relance du domaine.
fn row_vers_follow_up(row: &rusqlite::Row) -> rusqlite::Result<FollowUp> {
    Ok(FollowUp {
        id: uuid_column(row, 0)?,
        application_id: uuid_column(row, 1)?,
        application_job_title: row.get(2)?,
        company_name: row.get(3)?,
        follow_up_date: row.get(4)?,
        channel: row.get(5)?,
        notes: row.get(6)?,
        created_at: row.get(7)?,
    })
}

/// Lit une requête de relances jusqu'au bout.
fn collecter(
    conn: &rusqlite::Connection,
    sql: &str,
    params: &[&dyn rusqlite::ToSql],
) -> AppResult<Vec<FollowUp>> {
    let mut query = conn
        .prepare(sql)
        .map_err(|e| translate_error(e, "relances"))?;
    let rows = query
        .query_map(params, row_vers_follow_up)
        .map_err(|e| translate_error(e, "relances"))?;
    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| translate_error(e, "relances"))?);
    }
    Ok(items)
}

impl FollowUpRepository for SqliteFollowUpRepository {
    fn list(&self) -> AppResult<Vec<FollowUp>> {
        let conn = connection(&self.pool)?;
        collecter(
            &conn,
            &format!("SELECT {COLUMNS} {FROM_SQL} ORDER BY r.follow_up_date DESC"),
            &[],
        )
    }

    fn list_between(&self, from: &str, to: &str) -> AppResult<Vec<FollowUp>> {
        let conn = connection(&self.pool)?;
        collecter(
            &conn,
            &format!(
                "SELECT {COLUMNS} {FROM_SQL} \
                 WHERE r.follow_up_date >= ?1 AND r.follow_up_date <= ?2 \
                 ORDER BY r.follow_up_date ASC"
            ),
            &[&from, &to],
        )
    }

    fn create(&self, input: &NewFollowUp) -> AppResult<FollowUp> {
        let conn = connection(&self.pool)?;
        let id = Uuid::new_v4();
        conn.execute(
            "INSERT INTO follow_ups (id, application_id, follow_up_date, type, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                id.to_string(),
                input.application_id.to_string(),
                input.follow_up_date,
                input.channel,
                input.notes,
                now_iso(),
            ],
        )
        .map_err(|e| {
            translate_constraint(
                e,
                "La candidature liée à cette relance est introuvable",
                "relance",
            )
        })?;
        self.get(id)
    }

    fn update(&self, id: Uuid, input: &NewFollowUp) -> AppResult<FollowUp> {
        let conn = connection(&self.pool)?;
        let modifiees = conn
            .execute(
                "UPDATE follow_ups SET application_id = ?2, follow_up_date = ?3, type = ?4,
                    notes = ?5
                 WHERE id = ?1",
                rusqlite::params![
                    id.to_string(),
                    input.application_id.to_string(),
                    input.follow_up_date,
                    input.channel,
                    input.notes,
                ],
            )
            .map_err(|e| {
                translate_constraint(
                    e,
                    "La candidature liée à cette relance est introuvable",
                    "relance",
                )
            })?;
        if modifiees == 0 {
            return Err(AppError::NotFound(format!("relance {id}")));
        }
        self.get(id)
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = connection(&self.pool)?;
        let deleted = conn
            .execute("DELETE FROM follow_ups WHERE id = ?1", [id.to_string()])
            .map_err(|e| translate_error(e, "relance"))?;
        if deleted == 0 {
            return Err(AppError::NotFound(format!("relance {id}")));
        }
        Ok(())
    }
}

impl SqliteFollowUpRepository {
    /// Relit une relance après écriture.
    fn get(&self, id: Uuid) -> AppResult<FollowUp> {
        let conn = connection(&self.pool)?;
        collecter(
            &conn,
            &format!("SELECT {COLUMNS} {FROM_SQL} WHERE r.id = ?1"),
            &[&id.to_string()],
        )?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound(format!("relance {id}")))
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
