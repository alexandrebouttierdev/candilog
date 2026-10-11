//! Application transactionnelle des changements distants et résolution des conflits.
//!
//! Un lot (ou un instantané) s'applique dans **une** transaction, curseur compris : une
//! coupure au milieu l'annule en entier et le lot sera rejoué au prochain cycle. Les
//! déclencheurs de journalisation sont neutralisés le temps de l'application (`applying`),
//! pour ne pas renvoyer en écho ce qui vient d'arriver.

use crate::core::database::helpers::now_iso;
use crate::core::errors::{AppError, AppResult};
use crate::features::sync::domain::entities::{self, OnDelete, SyncedTable};
use crate::features::sync::domain::merge::{
    decide, ConflictKind, Decision, FieldConflict, LocalView, RecordMeta,
};
use crate::features::sync::domain::model::ConflictChoice;
use crate::features::sync::domain::protocol::{RecordChange, Row};
use crate::features::sync::domain::version::VersionVector;
use crate::features::sync::infrastructure::rows;
use crate::features::sync::infrastructure::store::{
    clear_sync_tables, SqliteSyncStore, SyncConfig,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Unité d'application : un lot d'une installation, ou un instantané.
#[derive(Debug, Clone, Copy)]
pub enum Unit<'a> {
    Batch {
        installation_id: &'a str,
        seq: u64,
        changes: &'a [RecordChange],
    },
    Snapshot {
        covered: &'a VersionVector,
        records: &'a [RecordChange],
        /// Retirer les enregistrements locaux absents de l'instantané qu'il a déjà vus :
        /// leur marqueur de suppression a dépassé la rétention.
        prune_absent: bool,
    },
}

/// Bilan d'une application.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ApplyReport {
    pub applied: u32,
    pub conflicts: u32,
    pub deferred: u32,
    pub profile_changed: bool,
}

impl ApplyReport {
    fn add(&mut self, outcome: Outcome, entity: &str) {
        match outcome {
            Outcome::Applied => {
                self.applied += 1;
                if entity == entities::PROFILE.name {
                    self.profile_changed = true;
                }
            }
            Outcome::Conflict => self.conflicts += 1,
            Outcome::Deferred => self.deferred += 1,
            Outcome::Ignored => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Applied,
    Ignored,
    Conflict,
    Deferred,
}

/// Version distante conservée avec un conflit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConflictRemote {
    pub version: VersionVector,
    pub columns: BTreeMap<String, VersionVector>,
    pub data: Option<Row>,
}

/// Conflit enregistré, avec l'état local courant pour l'affichage.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredConflict {
    pub id: String,
    pub entity: String,
    pub record_id: String,
    pub kind: ConflictKind,
    pub remote: ConflictRemote,
    pub fields: Vec<FieldConflict>,
    pub detected_at: String,
    pub local: Option<Row>,
}

fn connection(
    store: &SqliteSyncStore,
) -> AppResult<r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>> {
    store
        .pool()
        .get()
        .map_err(|error| AppError::Database(error.to_string()))
}

/// Exécute `work` dans une transaction où la journalisation est neutralisée.
fn applying<T>(
    conn: &mut Connection,
    work: impl FnOnce(&Connection) -> AppResult<T>,
) -> AppResult<T> {
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT OR IGNORE INTO sync_flags (name) VALUES ('applying')",
        [],
    )?;
    let result = work(&tx)?;
    tx.execute("DELETE FROM sync_flags WHERE name = 'applying'", [])?;
    tx.commit()?;
    Ok(result)
}

/// Ordre d'application : créations et modifications des parents vers les enfants (une
/// version qui cesse d'être courante avant celle qui le devient), puis suppressions des
/// enfants vers les parents.
fn ordered(changes: &[RecordChange]) -> Vec<&RecordChange> {
    let current = |change: &RecordChange| {
        change
            .data
            .as_ref()
            .and_then(|data| data.get("is_current"))
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0)
    };
    let mut upserts: Vec<&RecordChange> = changes.iter().filter(|c| !c.is_tombstone()).collect();
    upserts.sort_by_key(|change| (entities::rank(&change.entity), current(change)));
    let mut tombstones: Vec<&RecordChange> = changes.iter().filter(|c| c.is_tombstone()).collect();
    tombstones.sort_by_key(|change| std::cmp::Reverse(entities::rank(&change.entity)));
    upserts.into_iter().chain(tombstones).collect()
}

impl SqliteSyncStore {
    /// Applique une unité reçue et avance le curseur correspondant, atomiquement.
    ///
    /// # Errors
    /// Erreur SQLite : la transaction est annulée et l'unité sera rejouée.
    pub fn apply_unit(&self, unit: Unit<'_>) -> AppResult<ApplyReport> {
        let mut conn = connection(self)?;
        applying(&mut conn, |tx| apply_unit_in(tx, unit))
    }

    /// Rattache l'ordinateur à un espace **en remplaçant** ses données par celles de
    /// l'espace, en une seule transaction : si l'application échoue, rien n'est perdu.
    ///
    /// # Errors
    /// Erreur SQLite : la base reste dans son état antérieur, non rattachée.
    pub fn attach_replacing(
        &self,
        config: &SyncConfig,
        units: &[(String, u64, Vec<RecordChange>)],
        snapshot: Option<(&VersionVector, &[RecordChange])>,
    ) -> AppResult<ApplyReport> {
        let mut conn = connection(self)?;
        let tx = conn.transaction()?;
        clear_sync_tables(&tx)?;
        tx.execute(
            "INSERT INTO sync_state (key, value) VALUES ('config', ?1), ('next_seq', '1')",
            [rows::json_text(config)?],
        )?;
        tx.execute(
            "INSERT INTO sync_flags (name) VALUES ('tracking'), ('applying')",
            [],
        )?;
        for table in entities::SYNCED_TABLES.iter().rev() {
            tx.execute(&format!("DELETE FROM {}", table.name), [])?;
        }
        let mut report = ApplyReport::default();
        if let Some((covered, records)) = snapshot {
            let partial = apply_unit_in(
                &tx,
                Unit::Snapshot {
                    covered,
                    records,
                    prune_absent: false,
                },
            )?;
            merge_reports(&mut report, partial);
        }
        for (installation_id, seq, changes) in units {
            let partial = apply_unit_in(
                &tx,
                Unit::Batch {
                    installation_id,
                    seq: *seq,
                    changes,
                },
            )?;
            merge_reports(&mut report, partial);
        }
        tx.execute("DELETE FROM sync_flags WHERE name = 'applying'", [])?;
        tx.commit()?;
        Ok(report)
    }

    /// Réessaie les changements en attente d'un parent.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn retry_deferred(&self) -> AppResult<ApplyReport> {
        let mut conn = connection(self)?;
        applying(&mut conn, |tx| {
            let mut statement = tx.prepare("SELECT change FROM sync_deferred")?;
            let texts = statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            drop(statement);
            let changes = texts
                .iter()
                .map(|text| rows::json_parse::<RecordChange>(text))
                .collect::<AppResult<Vec<_>>>()?;
            let mut report = ApplyReport::default();
            for change in ordered(&changes) {
                let outcome = apply_change(tx, change)?;
                report.add(outcome, &change.entity);
            }
            Ok(report)
        })
    }

    /// Conflits en attente, avec l'état local courant.
    ///
    /// # Errors
    /// Erreur SQLite ou conflit illisible.
    pub fn conflicts(&self) -> AppResult<Vec<StoredConflict>> {
        let conn = connection(self)?;
        let mut statement = conn.prepare(
            "SELECT id, entity, record_id, kind, remote, fields, detected_at
             FROM sync_conflicts ORDER BY detected_at, id",
        )?;
        let raw = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        raw.into_iter()
            .map(
                |(id, entity, record_id, kind, remote, fields, detected_at)| {
                    let table = entities::table(&entity).ok_or_else(|| {
                        AppError::Validation("conflit sur une table inconnue".into())
                    })?;
                    Ok(StoredConflict {
                        local: rows::read_row(&conn, table, &record_id)?,
                        id,
                        entity,
                        record_id,
                        kind: ConflictKind::parse(&kind).unwrap_or(ConflictKind::Edit),
                        remote: rows::json_parse(&remote)?,
                        fields: rows::json_parse(&fields)?,
                        detected_at,
                    })
                },
            )
            .collect()
    }

    /// Applique la décision de l'utilisateur sur un conflit.
    ///
    /// Quelle que soit la décision, l'enregistrement est republié avec une version qui a vu
    /// les deux versions en présence : les autres ordinateurs, qui voient le même conflit,
    /// adoptent la décision au lieu de devoir trancher à leur tour.
    ///
    /// # Errors
    /// `NotFound` si le conflit n'existe plus ; `Validation` si la suppression demandée est
    /// impossible (entreprise encore liée à des candidatures).
    pub fn resolve_conflict(&self, conflict_id: &str, choice: ConflictChoice) -> AppResult<()> {
        let mut conn = connection(self)?;
        let tx = conn.transaction()?;
        let conflict = read_conflict_by_id(&tx, conflict_id)?
            .ok_or_else(|| AppError::NotFound("conflit".into()))?;
        let table = entities::table(&conflict.entity)
            .ok_or_else(|| AppError::Validation("conflit sur une table inconnue".into()))?;
        let id = conflict.record_id.as_str();
        let row = rows::read_row(&tx, table, id)?;
        let mut meta = rows::read_meta(&tx, table.name, id)?.unwrap_or_default();
        meta.version.merge(&conflict.remote.version);

        match (conflict.kind, choice) {
            (ConflictKind::Edit, ConflictChoice::KeepLocal) => {
                for field in &conflict.fields {
                    widen_column(&mut meta, &field.column, &field.remote_version);
                    meta.forced.insert(field.column.clone());
                }
                rows::write_meta(&tx, table.name, id, &meta)?;
            }
            (ConflictKind::Edit, ConflictChoice::TakeRemote) => {
                if let Some(mut row) = row {
                    let mut base = meta.base.clone().unwrap_or_default();
                    for field in &conflict.fields {
                        row.insert(field.column.clone(), field.remote.clone());
                        base.insert(field.column.clone(), field.remote.clone());
                        widen_column(&mut meta, &field.column, &field.remote_version);
                        meta.forced.insert(field.column.clone());
                    }
                    meta.base = Some(base);
                    with_flag(&tx, |tx| write_with_constraints(tx, table, id, row, meta))?;
                } else {
                    rows::write_meta(&tx, table.name, id, &meta)?;
                }
            }
            (ConflictKind::DeletedRemotely, ConflictChoice::KeepLocal) => {
                meta.deleted = false;
                meta.forced = non_key_columns(table);
                rows::write_meta(&tx, table.name, id, &meta)?;
            }
            (ConflictKind::DeletedRemotely, ConflictChoice::TakeRemote) => {
                rows::write_meta(&tx, table.name, id, &meta)?;
                // Journalisation active : les enfants supprimés en cascade partent aussi.
                rows::delete_row(&tx, table, id).map_err(|error| match error {
                    AppError::Database(_) => AppError::Validation(
                        "Cette fiche est encore liée à d'autres données (par exemple une \
                         entreprise à ses candidatures) : supprimez-les d'abord."
                            .into(),
                    ),
                    other => other,
                })?;
            }
            (ConflictKind::DeletedLocally, ConflictChoice::KeepLocal) => {
                meta.deleted = true;
                meta.forced.clear();
                rows::write_meta(&tx, table.name, id, &meta)?;
            }
            (ConflictKind::DeletedLocally, ConflictChoice::TakeRemote) => {
                let data = conflict
                    .remote
                    .data
                    .clone()
                    .ok_or_else(|| AppError::Validation("version distante absente".into()))?;
                with_flag(&tx, |tx| {
                    if ensure_parents(tx, table, &data)? == ParentState::Missing {
                        return Err(AppError::Validation(
                            "La fiche parente n'existe plus sur cet ordinateur : restaurez-la \
                             d'abord."
                                .into(),
                        ));
                    }
                    let restored = RecordMeta {
                        version: meta.version.clone(),
                        columns: conflict.remote.columns.clone(),
                        base: Some(data.clone()),
                        forced: non_key_columns(table),
                        deleted: false,
                    };
                    write_with_constraints(tx, table, id, data, restored)
                })?;
            }
        }
        rows::mark_dirty(&tx, table.name, id)?;
        tx.execute("DELETE FROM sync_conflicts WHERE id = ?1", [conflict_id])?;
        tx.commit()?;
        Ok(())
    }
}

fn merge_reports(total: &mut ApplyReport, part: ApplyReport) {
    total.applied += part.applied;
    total.conflicts += part.conflicts;
    total.deferred += part.deferred;
    total.profile_changed |= part.profile_changed;
}

fn with_flag<T>(conn: &Connection, work: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
    conn.execute(
        "INSERT OR IGNORE INTO sync_flags (name) VALUES ('applying')",
        [],
    )?;
    let result = work(conn);
    conn.execute("DELETE FROM sync_flags WHERE name = 'applying'", [])?;
    result
}

fn non_key_columns(table: &SyncedTable) -> BTreeSet<String> {
    table
        .columns
        .iter()
        .filter(|column| **column != table.key)
        .map(|column| (*column).to_owned())
        .collect()
}

fn widen_column(meta: &mut RecordMeta, column: &str, version: &VersionVector) {
    meta.columns
        .entry(column.to_owned())
        .or_default()
        .merge(version);
}

fn apply_unit_in(tx: &Connection, unit: Unit<'_>) -> AppResult<ApplyReport> {
    let mut report = ApplyReport::default();
    match unit {
        Unit::Batch {
            installation_id,
            seq,
            changes,
        } => {
            for change in ordered(changes) {
                let outcome = apply_isolated(tx, change)?;
                report.add(outcome, &change.entity);
            }
            tx.execute(
                "INSERT INTO sync_cursors (installation_id, last_seq) VALUES (?1, ?2)
                 ON CONFLICT(installation_id) DO UPDATE SET
                    last_seq = max(last_seq, excluded.last_seq)",
                params![installation_id, i64::try_from(seq).unwrap_or(i64::MAX)],
            )?;
        }
        Unit::Snapshot {
            covered,
            records,
            prune_absent,
        } => {
            for change in ordered(records) {
                let outcome = apply_isolated(tx, change)?;
                report.add(outcome, &change.entity);
            }
            if prune_absent {
                prune(tx, covered, records, &mut report)?;
            }
            for (installation, seq) in covered.entries() {
                tx.execute(
                    "INSERT INTO sync_cursors (installation_id, last_seq) VALUES (?1, ?2)
                     ON CONFLICT(installation_id) DO UPDATE SET
                        last_seq = max(last_seq, excluded.last_seq)",
                    params![installation, i64::try_from(*seq).unwrap_or(i64::MAX)],
                )?;
            }
        }
    }
    Ok(report)
}

/// Applique un changement dans un point de sauvegarde : une contrainte refusée (référentiel
/// absent de cette version, valeur hors catalogue) met ce seul changement en attente au lieu
/// de bloquer tout le lot indéfiniment.
fn apply_isolated(tx: &Connection, change: &RecordChange) -> AppResult<Outcome> {
    tx.execute_batch("SAVEPOINT sync_change")?;
    match apply_change(tx, change) {
        Ok(outcome) => {
            tx.execute_batch("RELEASE sync_change")?;
            Ok(outcome)
        }
        Err(AppError::Database(message)) if message.contains("constraint") => {
            tx.execute_batch("ROLLBACK TO sync_change; RELEASE sync_change")?;
            tracing::warn!(entity = %change.entity, %message, "changement distant refusé par une contrainte");
            defer(tx, change)?;
            Ok(Outcome::Deferred)
        }
        Err(error) => {
            tx.execute_batch("ROLLBACK TO sync_change; RELEASE sync_change")?;
            Err(error)
        }
    }
}

fn defer(tx: &Connection, change: &RecordChange) -> AppResult<()> {
    tx.execute(
        "INSERT INTO sync_deferred (entity, record_id, change, received_at) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(entity, record_id) DO UPDATE SET change = excluded.change",
        params![
            change.entity,
            change.id,
            rows::json_text(change)?,
            now_iso()
        ],
    )?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParentState {
    Ready,
    Missing,
}

/// Vérifie que les parents d'un enregistrement existent. Un parent supprimé ici mais
/// encore utilisé ailleurs est restauré depuis son dernier contenu connu : perdre la
/// candidature créée sur l'autre ordinateur serait pire que de faire réapparaître son
/// entreprise. Un parent jamais vu est attendu (changement différé).
fn ensure_parents(tx: &Connection, table: &SyncedTable, data: &Row) -> AppResult<ParentState> {
    for link in table.parents {
        let Some(parent_id) = data.get(link.column).and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Some(parent) = entities::table(link.parent) else {
            continue;
        };
        if rows::read_row(tx, parent, parent_id)?.is_some() {
            continue;
        }
        let meta = rows::read_meta(tx, parent.name, parent_id)?;
        let Some((mut meta, base)) =
            meta.and_then(|meta| meta.base.clone().map(|base| (meta, base)))
        else {
            return Ok(ParentState::Missing);
        };
        if ensure_parents(tx, parent, &base)? == ParentState::Missing {
            return Ok(ParentState::Missing);
        }
        tracing::info!(
            entity = parent.name,
            "parent supprimé ici restauré pour un enregistrement distant"
        );
        rows::upsert_row(tx, parent, &base)?;
        meta.deleted = false;
        meta.forced = non_key_columns(parent);
        rows::write_meta(tx, parent.name, parent_id, &meta)?;
        rows::mark_dirty(tx, parent.name, parent_id)?;
    }
    Ok(ParentState::Ready)
}

/// Un profil jamais rempli compte comme absent : il ne doit pas entrer en conflit avec le
/// profil d'un autre ordinateur.
fn effective_row(table: &SyncedTable, row: Option<Row>, meta: Option<&RecordMeta>) -> Option<Row> {
    if table.name == entities::PROFILE.name && meta.is_none() {
        let empty = row
            .as_ref()
            .and_then(|row| row.get("data"))
            .and_then(serde_json::Value::as_str)
            .is_none_or(|data| matches!(data.trim(), "" | "{}"));
        if empty {
            return None;
        }
    }
    row
}

fn apply_change(tx: &Connection, change: &RecordChange) -> AppResult<Outcome> {
    let table = entities::table(&change.entity)
        .ok_or_else(|| AppError::Validation(format!("table inconnue : {}", change.entity)))?;
    let id = change.id.as_str();
    tx.execute(
        "DELETE FROM sync_deferred WHERE entity = ?1 AND record_id = ?2",
        params![table.name, id],
    )?;
    if let Some(data) = &change.data {
        if ensure_parents(tx, table, data)? == ParentState::Missing {
            defer(tx, change)?;
            return Ok(Outcome::Deferred);
        }
    }
    let meta = rows::read_meta(tx, table.name, id)?;
    let row = effective_row(table, rows::read_row(tx, table, id)?, meta.as_ref());
    let dirty = rows::is_dirty(tx, table.name, id)?;
    let previous = read_conflict(tx, table.name, id)?;
    let local = LocalView {
        row: row.as_ref(),
        meta: meta.as_ref(),
        dirty,
    };

    match decide(table, local, change) {
        Decision::Ignore => Ok(Outcome::Ignored),
        Decision::Write { row, meta } => {
            let kept = remaining_fields(previous.as_ref(), &meta, &row);
            write_with_constraints(tx, table, id, row, meta.clone())?;
            settle_previous(tx, previous, kept, &meta)?;
            Ok(Outcome::Applied)
        }
        Decision::Delete { meta } => {
            if delete_remote(tx, table, id, &meta)? {
                tx.execute(
                    "DELETE FROM sync_conflicts WHERE entity = ?1 AND record_id = ?2",
                    params![table.name, id],
                )?;
                Ok(Outcome::Applied)
            } else {
                store_conflict(
                    tx,
                    table,
                    id,
                    ConflictKind::DeletedRemotely,
                    change,
                    Vec::new(),
                )?;
                Ok(Outcome::Conflict)
            }
        }
        Decision::RecordTombstone { meta } => {
            rows::write_meta(tx, table.name, id, &meta)?;
            tx.execute(
                "DELETE FROM sync_conflicts WHERE entity = ?1 AND record_id = ?2",
                params![table.name, id],
            )?;
            Ok(Outcome::Applied)
        }
        Decision::Conflict {
            kind,
            fields,
            partial,
        } => {
            let mut fields = fields;
            if let Some((row, meta)) = partial {
                for field in remaining_fields(previous.as_ref(), &meta, &row) {
                    if !fields.iter().any(|known| known.column == field.column) {
                        fields.push(field);
                    }
                }
                write_with_constraints(tx, table, id, row, meta)?;
            }
            store_conflict(tx, table, id, kind, change, fields)?;
            Ok(Outcome::Conflict)
        }
    }
}

/// Colonnes d'un conflit antérieur toujours en désaccord après une nouvelle écriture.
fn remaining_fields(
    previous: Option<&StoredConflict>,
    meta: &RecordMeta,
    row: &Row,
) -> Vec<FieldConflict> {
    let Some(previous) = previous.filter(|conflict| conflict.kind == ConflictKind::Edit) else {
        return Vec::new();
    };
    let null = serde_json::Value::Null;
    previous
        .fields
        .iter()
        .filter(|field| {
            let settled = meta
                .columns
                .get(&field.column)
                .is_some_and(|version| field.remote_version.is_covered_by(version));
            !settled && row.get(&field.column).unwrap_or(&null) != &field.remote
        })
        .cloned()
        .collect()
}

fn settle_previous(
    tx: &Connection,
    previous: Option<StoredConflict>,
    kept: Vec<FieldConflict>,
    meta: &RecordMeta,
) -> AppResult<()> {
    let Some(previous) = previous else {
        return Ok(());
    };
    let resolved = match previous.kind {
        ConflictKind::Edit => kept.is_empty(),
        ConflictKind::DeletedRemotely | ConflictKind::DeletedLocally => {
            previous.remote.version.is_covered_by(&meta.version)
        }
    };
    if resolved {
        tx.execute("DELETE FROM sync_conflicts WHERE id = ?1", [&previous.id])?;
    } else if previous.kind == ConflictKind::Edit {
        tx.execute(
            "UPDATE sync_conflicts SET fields = ?1 WHERE id = ?2",
            params![rows::json_text(&kept)?, previous.id],
        )?;
    }
    Ok(())
}

fn store_conflict(
    tx: &Connection,
    table: &SyncedTable,
    id: &str,
    kind: ConflictKind,
    change: &RecordChange,
    fields: Vec<FieldConflict>,
) -> AppResult<()> {
    let remote = ConflictRemote {
        version: change.version.clone(),
        columns: change.columns.clone(),
        data: change.data.clone(),
    };
    tx.execute(
        "INSERT INTO sync_conflicts (id, entity, record_id, kind, remote, fields, detected_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(entity, record_id) DO UPDATE SET
            kind = excluded.kind, remote = excluded.remote, fields = excluded.fields",
        params![
            uuid::Uuid::new_v4().to_string(),
            table.name,
            id,
            kind.as_str(),
            rows::json_text(&remote)?,
            rows::json_text(&fields)?,
            now_iso()
        ],
    )?;
    Ok(())
}

fn read_conflict_where(
    conn: &Connection,
    clause: &str,
    values: &[&dyn rusqlite::ToSql],
) -> AppResult<Option<StoredConflict>> {
    let sql = format!(
        "SELECT id, entity, record_id, kind, remote, fields, detected_at
         FROM sync_conflicts WHERE {clause}"
    );
    let raw = conn
        .query_row(&sql, values, |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .optional()?;
    raw.map(
        |(id, entity, record_id, kind, remote, fields, detected_at)| {
            Ok(StoredConflict {
                id,
                entity,
                record_id,
                kind: ConflictKind::parse(&kind).unwrap_or(ConflictKind::Edit),
                remote: rows::json_parse(&remote)?,
                fields: rows::json_parse(&fields)?,
                detected_at,
                local: None,
            })
        },
    )
    .transpose()
}

fn read_conflict(conn: &Connection, entity: &str, id: &str) -> AppResult<Option<StoredConflict>> {
    read_conflict_where(conn, "entity = ?1 AND record_id = ?2", &[&entity, &id])
}

fn read_conflict_by_id(conn: &Connection, conflict_id: &str) -> AppResult<Option<StoredConflict>> {
    read_conflict_where(conn, "id = ?1", &[&conflict_id])
}

/// Supprime localement un enregistrement supprimé ailleurs, si rien ne s'y oppose.
///
/// Des enfants encore présents en cascade — que l'auteur de la suppression ne connaissait
/// donc pas — bloquent la suppression : elle effacerait en silence des données créées ici.
fn delete_remote(
    tx: &Connection,
    table: &SyncedTable,
    id: &str,
    meta: &RecordMeta,
) -> AppResult<bool> {
    let mut detached = Vec::new();
    for (child, link) in entities::children_of(table.name) {
        let ids = rows::child_ids(tx, child, link.column, id)?;
        match link.on_delete {
            OnDelete::Cascade | OnDelete::Restrict if !ids.is_empty() => return Ok(false),
            OnDelete::SetNull => {
                detached.extend(ids.into_iter().map(|child_id| (child.name, child_id)))
            }
            _ => {}
        }
    }
    rows::delete_row(tx, table, id)?;
    rows::write_meta(tx, table.name, id, meta)?;
    // Lien rompu par `SET NULL` sur des enregistrements que l'autre ordinateur ne
    // connaissait pas : la rupture doit lui être publiée.
    for (entity, child_id) in detached {
        rows::mark_dirty(tx, entity, &child_id)?;
    }
    Ok(true)
}

/// Retire les enregistrements absents d'un instantané qui les avait déjà vus.
fn prune(
    tx: &Connection,
    covered: &VersionVector,
    records: &[RecordChange],
    report: &mut ApplyReport,
) -> AppResult<()> {
    let present: BTreeSet<(&str, &str)> = records
        .iter()
        .map(|change| (change.entity.as_str(), change.id.as_str()))
        .collect();
    for table in entities::SYNCED_TABLES.iter().rev() {
        let mut statement =
            tx.prepare("SELECT record_id FROM sync_records WHERE entity = ?1 AND deleted = 0")?;
        let ids = statement
            .query_map([table.name], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        drop(statement);
        for id in ids {
            if present.contains(&(table.name, id.as_str())) || rows::is_dirty(tx, table.name, &id)?
            {
                continue;
            }
            let Some(mut meta) = rows::read_meta(tx, table.name, &id)? else {
                continue;
            };
            if !meta.version.is_covered_by(covered) || !meta.forced.is_empty() {
                continue;
            }
            meta.deleted = true;
            meta.columns.clear();
            if rows::read_row(tx, table, &id)?.is_none() {
                rows::write_meta(tx, table.name, &id, &meta)?;
            } else if delete_remote(tx, table, &id, &meta)? {
                report.applied += 1;
            }
        }
    }
    Ok(())
}

/// Écrit une ligne en respectant les contraintes d'unicité que deux ordinateurs peuvent
/// enfreindre sans le savoir : numéro de candidature et version courante d'un document.
/// L'arbitrage est déterministe — le même sur chaque ordinateur — et la ligne déplacée est
/// republiée pour que tous convergent.
fn write_with_constraints(
    tx: &Connection,
    table: &SyncedTable,
    id: &str,
    mut row: Row,
    mut meta: RecordMeta,
) -> AppResult<()> {
    if table.name == entities::APPLICATIONS.name {
        settle_reference(tx, id, &mut row, &mut meta)?;
    }
    if table.name == entities::RESUME_VERSIONS.name || table.name == entities::COVER_LETTERS.name {
        settle_current(tx, table, id, &mut row, &mut meta)?;
    }
    rows::upsert_row(tx, table, &row)?;
    rows::write_meta(tx, table.name, id, &meta)
}

fn force_column(tx: &Connection, entity: &str, id: &str, column: &str) -> AppResult<()> {
    if let Some(mut meta) = rows::read_meta(tx, entity, id)? {
        meta.forced.insert(column.to_owned());
        rows::write_meta(tx, entity, id, &meta)?;
    }
    rows::mark_dirty(tx, entity, id)
}

fn next_reference(tx: &Connection) -> AppResult<i64> {
    let next: i64 = tx.query_row(
        "SELECT max(
            coalesce((SELECT CAST(kv_value AS INTEGER) FROM app_kv
                      WHERE kv_key = 'last_application_reference'), 0),
            coalesce((SELECT max(reference_number) FROM applications), 0)
         ) + 1",
        [],
        |row| row.get(0),
    )?;
    tx.execute(
        "INSERT OR REPLACE INTO app_kv (kv_key, kv_value)
         VALUES ('last_application_reference', CAST(?1 AS TEXT))",
        [next],
    )?;
    Ok(next)
}

fn settle_reference(
    tx: &Connection,
    id: &str,
    row: &mut Row,
    meta: &mut RecordMeta,
) -> AppResult<()> {
    let Some(reference) = row
        .get("reference_number")
        .and_then(serde_json::Value::as_i64)
    else {
        return Ok(());
    };
    // Le compteur ne redescend jamais sous un numéro reçu : il ne sera pas réattribué ici.
    tx.execute(
        "INSERT INTO app_kv (kv_key, kv_value) VALUES ('last_application_reference', CAST(?1 AS TEXT))
         ON CONFLICT(kv_key) DO UPDATE SET kv_value = CAST(max(CAST(kv_value AS INTEGER), ?1) AS TEXT)",
        [reference],
    )?;
    let holder: Option<String> = tx
        .query_row(
            "SELECT id FROM applications WHERE reference_number = ?1 AND id <> ?2",
            params![reference, id],
            |row| row.get(0),
        )
        .optional()?;
    let Some(holder) = holder else {
        return Ok(());
    };
    // Le plus petit identifiant garde le numéro, sur tous les ordinateurs.
    let fresh = next_reference(tx)?;
    if id < holder.as_str() {
        tx.execute(
            "UPDATE applications SET reference_number = ?1 WHERE id = ?2",
            params![fresh, holder],
        )?;
        force_column(tx, entities::APPLICATIONS.name, &holder, "reference_number")?;
    } else {
        row.insert("reference_number".into(), serde_json::json!(fresh));
        meta.forced.insert("reference_number".into());
        rows::mark_dirty(tx, entities::APPLICATIONS.name, id)?;
    }
    Ok(())
}

fn settle_current(
    tx: &Connection,
    table: &SyncedTable,
    id: &str,
    row: &mut Row,
    meta: &mut RecordMeta,
) -> AppResult<()> {
    if row.get("is_current").and_then(serde_json::Value::as_i64) != Some(1) {
        return Ok(());
    }
    let Some(document) = row.get("document_id").and_then(serde_json::Value::as_str) else {
        return Ok(());
    };
    let version = row
        .get("version_number")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(1);
    let holder: Option<(String, i64)> = tx
        .query_row(
            &format!(
                "SELECT id, version_number FROM {} WHERE document_id = ?1 AND is_current = 1 AND id <> ?2",
                table.name
            ),
            params![document, id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((holder, holder_version)) = holder else {
        return Ok(());
    };
    // La version au numéro le plus élevé reste courante ; aucune version n'est perdue.
    if (version, id) > (holder_version, holder.as_str()) {
        tx.execute(
            &format!("UPDATE {} SET is_current = 0 WHERE id = ?1", table.name),
            [&holder],
        )?;
        force_column(tx, table.name, &holder, "is_current")?;
    } else {
        row.insert("is_current".into(), serde_json::json!(0));
        meta.forced.insert("is_current".into());
        rows::mark_dirty(tx, table.name, id)?;
    }
    Ok(())
}
