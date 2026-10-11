//! Lecture et écriture génériques des lignes synchronisées.
//!
//! Les noms de tables et de colonnes interpolés ici viennent exclusivement de la
//! cartographie statique (`domain::entities`) ; aucune valeur reçue n'entre dans le SQL
//! autrement que par paramètre.

use crate::core::errors::{AppError, AppResult};
use crate::features::sync::domain::entities::SyncedTable;
use crate::features::sync::domain::merge::RecordMeta;
use crate::features::sync::domain::protocol::Row;
use rusqlite::types::Value as SqlValue;
use rusqlite::{params, Connection, OptionalExtension};

/// Valeur SQLite → valeur JSON.
pub fn to_json(value: SqlValue) -> serde_json::Value {
    match value {
        SqlValue::Null => serde_json::Value::Null,
        SqlValue::Integer(number) => serde_json::Value::from(number),
        SqlValue::Real(number) => serde_json::Number::from_f64(number)
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        SqlValue::Text(text) => serde_json::Value::String(text),
        // Aucune table synchronisée ne porte de BLOB ; une telle valeur ne quitte pas la base.
        SqlValue::Blob(_) => serde_json::Value::Null,
    }
}

/// Valeur JSON → valeur SQLite.
pub fn to_sql(value: &serde_json::Value) -> SqlValue {
    match value {
        serde_json::Value::Bool(flag) => SqlValue::Integer(i64::from(*flag)),
        serde_json::Value::Number(number) => number.as_i64().map_or_else(
            || SqlValue::Real(number.as_f64().unwrap_or_default()),
            SqlValue::Integer,
        ),
        serde_json::Value::String(text) => SqlValue::Text(text.clone()),
        serde_json::Value::Null | serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            SqlValue::Null
        }
    }
}

/// Paramètre d'identifiant pour une table.
pub fn key_param(table: &SyncedTable, id: &str) -> AppResult<SqlValue> {
    if table.integer_key {
        id.parse::<i64>()
            .map(SqlValue::Integer)
            .map_err(|_| AppError::Validation(format!("identifiant invalide : {id}")))
    } else {
        Ok(SqlValue::Text(id.to_owned()))
    }
}

/// Ligne courante d'un enregistrement.
pub fn read_row(conn: &Connection, table: &SyncedTable, id: &str) -> AppResult<Option<Row>> {
    let sql = format!(
        "SELECT {} FROM {} WHERE {} = ?1",
        table.columns.join(", "),
        table.name,
        table.key
    );
    let key = key_param(table, id)?;
    let row = conn
        .query_row(&sql, [key], |sql_row| {
            let mut row = Row::new();
            for (index, column) in table.columns.iter().enumerate() {
                row.insert((*column).to_owned(), to_json(sql_row.get(index)?));
            }
            Ok(row)
        })
        .optional()?;
    Ok(row)
}

/// Identifiants de toutes les lignes d'une table.
pub fn all_ids(conn: &Connection, table: &SyncedTable) -> AppResult<Vec<String>> {
    let sql = format!(
        "SELECT CAST({key} AS TEXT) FROM {name} ORDER BY {key}",
        key = table.key,
        name = table.name
    );
    let mut statement = conn.prepare(&sql)?;
    let ids = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids)
}

/// Crée ou remplace une ligne avec les colonnes présentes dans `row`.
pub fn upsert_row(conn: &Connection, table: &SyncedTable, row: &Row) -> AppResult<()> {
    let columns: Vec<&str> = table
        .columns
        .iter()
        .copied()
        .filter(|column| row.contains_key(*column))
        .collect();
    if !columns.contains(&table.key) {
        return Err(AppError::Validation("identifiant absent".into()));
    }
    let placeholders: Vec<String> = (1..=columns.len())
        .map(|index| format!("?{index}"))
        .collect();
    let updates: Vec<String> = columns
        .iter()
        .filter(|column| **column != table.key)
        .map(|column| format!("{column} = excluded.{column}"))
        .collect();
    let conflict = if updates.is_empty() {
        "DO NOTHING".to_owned()
    } else {
        format!("DO UPDATE SET {}", updates.join(", "))
    };
    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT({}) {}",
        table.name,
        columns.join(", "),
        placeholders.join(", "),
        table.key,
        conflict
    );
    let values: Vec<SqlValue> = columns
        .iter()
        .map(|column| row.get(*column).map_or(SqlValue::Null, to_sql))
        .collect();
    conn.execute(&sql, rusqlite::params_from_iter(values))?;
    Ok(())
}

/// Supprime une ligne.
pub fn delete_row(conn: &Connection, table: &SyncedTable, id: &str) -> AppResult<()> {
    let sql = format!("DELETE FROM {} WHERE {} = ?1", table.name, table.key);
    conn.execute(&sql, [key_param(table, id)?])?;
    Ok(())
}

/// Lignes enfants d'un parent par une clé étrangère.
pub fn child_ids(
    conn: &Connection,
    child: &SyncedTable,
    column: &str,
    parent_id: &str,
) -> AppResult<Vec<String>> {
    let sql = format!(
        "SELECT CAST({key} AS TEXT) FROM {name} WHERE {column} = ?1",
        key = child.key,
        name = child.name
    );
    let mut statement = conn.prepare(&sql)?;
    let ids = statement
        .query_map([parent_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids)
}

fn parse_json<T: serde::de::DeserializeOwned>(text: &str) -> AppResult<T> {
    serde_json::from_str(text).map_err(|error| AppError::Serialization(error.to_string()))
}

fn to_json_text<T: serde::Serialize>(value: &T) -> AppResult<String> {
    serde_json::to_string(value).map_err(|error| AppError::Serialization(error.to_string()))
}

/// Métadonnées de synchronisation d'un enregistrement.
pub fn read_meta(conn: &Connection, entity: &str, id: &str) -> AppResult<Option<RecordMeta>> {
    let raw = conn
        .query_row(
            "SELECT version, columns, base, forced, deleted FROM sync_records
             WHERE entity = ?1 AND record_id = ?2",
            params![entity, id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        )
        .optional()?;
    let Some((version, columns, base, forced, deleted)) = raw else {
        return Ok(None);
    };
    Ok(Some(RecordMeta {
        version: parse_json(&version)?,
        columns: parse_json(&columns)?,
        base: base.as_deref().map(parse_json).transpose()?,
        forced: parse_json(&forced)?,
        deleted: deleted != 0,
    }))
}

/// Enregistre les métadonnées de synchronisation d'un enregistrement.
pub fn write_meta(conn: &Connection, entity: &str, id: &str, meta: &RecordMeta) -> AppResult<()> {
    conn.execute(
        "INSERT INTO sync_records (entity, record_id, version, columns, base, forced, deleted, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(entity, record_id) DO UPDATE SET
            version = excluded.version, columns = excluded.columns, base = excluded.base,
            forced = excluded.forced, deleted = excluded.deleted, updated_at = excluded.updated_at",
        params![
            entity,
            id,
            to_json_text(&meta.version)?,
            to_json_text(&meta.columns)?,
            meta.base.as_ref().map(to_json_text).transpose()?,
            to_json_text(&meta.forced)?,
            i64::from(meta.deleted),
            crate::core::database::helpers::now_iso(),
        ],
    )?;
    Ok(())
}

/// L'enregistrement attend-il d'être publié ?
pub fn is_dirty(conn: &Connection, entity: &str, id: &str) -> AppResult<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sync_outbox WHERE entity = ?1 AND record_id = ?2)",
        params![entity, id],
        |row| row.get::<_, bool>(0),
    )?)
}

/// Inscrit explicitement un enregistrement au prochain lot (hors déclencheurs).
pub fn mark_dirty(conn: &Connection, entity: &str, id: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO sync_outbox (entity, record_id) VALUES (?1, ?2)",
        params![entity, id],
    )?;
    Ok(())
}

pub fn json_text<T: serde::Serialize>(value: &T) -> AppResult<String> {
    to_json_text(value)
}

pub fn json_parse<T: serde::de::DeserializeOwned>(text: &str) -> AppResult<T> {
    parse_json(text)
}
