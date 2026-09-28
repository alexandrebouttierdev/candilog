//! Dépôt `SQLite` des vues enregistrées.

use crate::core::database::helpers::{connection, now_iso, translate_error, uuid_column};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::views::domain::{NewSavedView, SavedView, SavedViewRepository};
use uuid::Uuid;

/// Implémentation `SQLite` du dépôt de vues.
pub struct SqliteSavedViewRepository {
    pub(crate) pool: SqlitePool,
}

impl SqliteSavedViewRepository {
    /// Construit le dépôt à partir du pool local.
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

const COLUMNS: &str = "id, name, filter, position, created_at, updated_at";

fn row_to_view(row: &rusqlite::Row) -> AppResult<SavedView> {
    let read_err = |e| translate_error(e, "vue");
    let filter: String = row.get(2).map_err(read_err)?;
    Ok(SavedView {
        id: uuid_column(row, 0).map_err(read_err)?,
        name: row.get(1).map_err(read_err)?,
        // Un filtre illisible (base modifiée à la main) est une erreur dite, pas une vue vide
        // qui montrerait silencieusement toutes les candidatures.
        filter: serde_json::from_str(&filter)
            .map_err(|_| AppError::Validation("Le filtre de cette vue est illisible.".into()))?,
        position: row.get(3).map_err(read_err)?,
        created_at: row.get(4).map_err(read_err)?,
        updated_at: row.get(5).map_err(read_err)?,
    })
}

fn filter_json(input: &NewSavedView) -> AppResult<String> {
    serde_json::to_string(&input.filter).map_err(|e| AppError::Serialization(e.to_string()))
}

impl SavedViewRepository for SqliteSavedViewRepository {
    fn list(&self) -> AppResult<Vec<SavedView>> {
        let conn = connection(&self.pool)?;
        let mut query = conn
            .prepare(&format!(
                "SELECT {COLUMNS} FROM saved_views ORDER BY position, created_at"
            ))
            .map_err(|e| translate_error(e, "vues"))?;
        let mut rows = query.query([]).map_err(|e| translate_error(e, "vues"))?;
        let mut views = Vec::new();
        while let Some(row) = rows.next().map_err(|e| translate_error(e, "vues"))? {
            views.push(row_to_view(row)?);
        }
        Ok(views)
    }

    fn create(&self, input: &NewSavedView) -> AppResult<SavedView> {
        let conn = connection(&self.pool)?;
        let id = Uuid::new_v4();
        let now = now_iso();
        conn.execute(
            "INSERT INTO saved_views (id, name, filter, position, created_at, updated_at)
             VALUES (?1, ?2, ?3,
                     (SELECT coalesce(max(position), 0) + 1 FROM saved_views), ?4, ?4)",
            rusqlite::params![id.to_string(), input.name.trim(), filter_json(input)?, now],
        )
        .map_err(|e| translate_error(e, "vue"))?;
        self.get(id)
    }

    fn update(&self, id: Uuid, input: &NewSavedView) -> AppResult<SavedView> {
        let conn = connection(&self.pool)?;
        let changed = conn
            .execute(
                "UPDATE saved_views SET name = ?2, filter = ?3, updated_at = ?4 WHERE id = ?1",
                rusqlite::params![
                    id.to_string(),
                    input.name.trim(),
                    filter_json(input)?,
                    now_iso()
                ],
            )
            .map_err(|e| translate_error(e, "vue"))?;
        if changed == 0 {
            return Err(AppError::NotFound(format!("vue {id}")));
        }
        self.get(id)
    }

    fn get(&self, id: Uuid) -> AppResult<SavedView> {
        let conn = connection(&self.pool)?;
        let mut query = conn
            .prepare(&format!("SELECT {COLUMNS} FROM saved_views WHERE id = ?1"))
            .map_err(|e| translate_error(e, "vue"))?;
        let mut rows = query
            .query([id.to_string()])
            .map_err(|e| translate_error(e, "vue"))?;
        match rows.next().map_err(|e| translate_error(e, "vue"))? {
            Some(row) => row_to_view(row),
            None => Err(AppError::NotFound(format!("vue {id}"))),
        }
    }

    fn delete(&self, id: Uuid) -> AppResult<()> {
        let conn = connection(&self.pool)?;
        let deleted = conn
            .execute("DELETE FROM saved_views WHERE id = ?1", [id.to_string()])
            .map_err(|e| translate_error(e, "vue"))?;
        if deleted == 0 {
            return Err(AppError::NotFound(format!("vue {id}")));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
