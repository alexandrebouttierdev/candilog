//! État local de la synchronisation dans SQLite : rattachement, journal, lots publiés.

use crate::core::database::helpers::now_iso;
use crate::core::database::{SqlitePool, LATEST_SCHEMA_VERSION};
use crate::core::errors::{AppError, AppResult};
use crate::features::sync::domain::entities::{self, SyncedTable, SYNCED_TABLES};
use crate::features::sync::domain::merge::{pending_columns, RecordMeta};
use crate::features::sync::domain::model::SyncProviderKind;
use crate::features::sync::domain::protocol::{
    self, Attachment, ChangeBatch, RecordChange, Row, Snapshot, FORMAT_VERSION,
};
use crate::features::sync::domain::version::VersionVector;
use crate::features::sync::infrastructure::rows;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Configuration de la synchronisation propre à cet ordinateur.
///
/// Elle vit dans `sync_state` et non dans les réglages : restaurer une sauvegarde ou
/// remettre l'application à zéro doit la détacher en même temps que l'état qui l'accompagne,
/// sans quoi un ordinateur reprendrait les compteurs d'un autre.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncConfig {
    pub provider: SyncProviderKind,
    /// Chemin du dossier choisi (mode dossier personnalisé).
    pub folder: Option<String>,
    pub space_id: Option<String>,
    pub installation_id: Option<String>,
    pub auto_sync: bool,
}

impl SyncConfig {
    /// Rattaché à un espace et prêt à synchroniser.
    #[must_use]
    pub fn attached(&self) -> Option<(&str, &str)> {
        match (&self.space_id, &self.installation_id) {
            (Some(space), Some(installation)) if self.provider != SyncProviderKind::Local => {
                Some((space.as_str(), installation.as_str()))
            }
            _ => None,
        }
    }
}

/// Fichier joint à téléverser avant le lot qui le référence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobUpload {
    pub blob: String,
    pub file: String,
}

/// Lot prêt à l'envoi, conservé jusqu'à confirmation du fournisseur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedBatch {
    pub seq: u64,
    pub name: String,
    pub payload: Vec<u8>,
    pub blobs: Vec<BlobUpload>,
    pub records: u32,
}

/// Compteurs affichés par l'écran de synchronisation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SyncCounts {
    pub pending: u32,
    pub conflicts: u32,
    pub waiting: u32,
}

const CONFIG_KEY: &str = "config";
const NEXT_SEQ_KEY: &str = "next_seq";
const LAST_SUCCESS_KEY: &str = "last_success_at";
const OWN_SNAPSHOTS_KEY: &str = "own_snapshots";
/// Nombre de lots publiés depuis le dernier instantané.
const SINCE_SNAPSHOT_KEY: &str = "batches_since_snapshot";

/// Instantané publié par cet ordinateur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnSnapshot {
    pub seq: u64,
    pub covered_self: u64,
    pub created_at: String,
}

/// Accès SQLite de la synchronisation.
#[derive(Clone)]
pub struct SqliteSyncStore {
    pool: SqlitePool,
}

fn connection(
    pool: &SqlitePool,
) -> AppResult<r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>> {
    pool.get()
        .map_err(|error| AppError::Database(error.to_string()))
}

fn get_state(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT value FROM sync_state WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .optional()?)
}

fn set_state(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO sync_state (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub use crate::core::backup::clear_sync_state as clear_sync_tables;

impl SqliteSyncStore {
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    #[must_use]
    pub const fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// # Errors
    /// Erreur SQLite ou configuration illisible.
    pub fn config(&self) -> AppResult<SyncConfig> {
        let conn = connection(&self.pool)?;
        get_state(&conn, CONFIG_KEY)?
            .map_or_else(|| Ok(SyncConfig::default()), |text| rows::json_parse(&text))
    }

    /// # Errors
    /// Erreur SQLite.
    pub fn save_config(&self, config: &SyncConfig) -> AppResult<()> {
        let conn = connection(&self.pool)?;
        set_state(&conn, CONFIG_KEY, &rows::json_text(config)?)
    }

    /// # Errors
    /// Erreur SQLite.
    pub fn last_success(&self) -> AppResult<Option<String>> {
        let conn = connection(&self.pool)?;
        get_state(&conn, LAST_SUCCESS_KEY)
    }

    /// # Errors
    /// Erreur SQLite.
    pub fn record_success(&self, at: &str) -> AppResult<()> {
        let conn = connection(&self.pool)?;
        set_state(&conn, LAST_SUCCESS_KEY, at)
    }

    /// Cet ordinateur contient-il des données utilisateur ?
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn local_has_data(&self) -> AppResult<bool> {
        let conn = connection(&self.pool)?;
        for table in SYNCED_TABLES {
            let present: bool = if table.name == entities::PROFILE.name {
                conn.query_row(
                    "SELECT EXISTS (SELECT 1 FROM profile WHERE data NOT IN ('{}', ''))",
                    [],
                    |row| row.get(0),
                )?
            } else {
                conn.query_row(
                    &format!("SELECT EXISTS (SELECT 1 FROM {})", table.name),
                    [],
                    |row| row.get(0),
                )?
            };
            if present {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Rattache l'ordinateur à un espace : état remis à neuf, journalisation activée et,
    /// si demandé, toutes les données locales inscrites au premier lot.
    ///
    /// # Errors
    /// Erreur SQLite ; rien n'est modifié en cas d'échec.
    pub fn attach(&self, config: &SyncConfig, publish_local_data: bool) -> AppResult<()> {
        let mut conn = connection(&self.pool)?;
        let tx = conn.transaction()?;
        clear_sync_tables(&tx)?;
        set_state(&tx, CONFIG_KEY, &rows::json_text(config)?)?;
        set_state(&tx, NEXT_SEQ_KEY, "1")?;
        tx.execute("INSERT INTO sync_flags (name) VALUES ('tracking')", [])?;
        if publish_local_data {
            enqueue_all(&tx)?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Détache l'ordinateur de son espace. Les données locales ne sont pas touchées.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn detach(&self) -> AppResult<()> {
        let mut conn = connection(&self.pool)?;
        let tx = conn.transaction()?;
        clear_sync_tables(&tx)?;
        tx.commit()?;
        Ok(())
    }

    /// Remplace l'identité de cette installation tout en gardant ce qui est connu de
    /// l'espace : utilisé quand la base a été copiée depuis un autre ordinateur. Un lot
    /// préparé sous l'ancienne identité est abandonné et ses enregistrements réinscrits.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn renew_installation(&self, installation_id: &str) -> AppResult<()> {
        let mut conn = connection(&self.pool)?;
        let tx = conn.transaction()?;
        let pending: Vec<Vec<u8>> = {
            let mut statement = tx.prepare("SELECT payload FROM sync_pending")?;
            let rows = statement
                .query_map([], |row| row.get::<_, Vec<u8>>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            rows
        };
        if !pending.is_empty() {
            // Le contenu n'est pas relu : toutes les données sont réinscrites, ce qui
            // republie au pire des valeurs déjà connues.
            enqueue_all(&tx)?;
        }
        tx.execute("DELETE FROM sync_pending", [])?;
        let mut config: SyncConfig = get_state(&tx, CONFIG_KEY)?
            .map_or_else(|| Ok(SyncConfig::default()), |text| rows::json_parse(&text))?;
        config.installation_id = Some(installation_id.to_owned());
        set_state(&tx, CONFIG_KEY, &rows::json_text(&config)?)?;
        set_state(&tx, NEXT_SEQ_KEY, "1")?;
        tx.execute(
            "DELETE FROM sync_state WHERE key IN (?1, ?2)",
            params![OWN_SNAPSHOTS_KEY, SINCE_SNAPSHOT_KEY],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Compteurs de l'écran : modifications en attente, conflits, changements différés.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn counts(&self) -> AppResult<SyncCounts> {
        let conn = connection(&self.pool)?;
        let outbox: i64 = conn.query_row(
            "SELECT count(*) FROM (SELECT DISTINCT entity, record_id FROM sync_outbox)",
            [],
            |row| row.get(0),
        )?;
        let pending: i64 = conn.query_row(
            "SELECT coalesce(sum(records), 0) FROM sync_pending",
            [],
            |row| row.get(0),
        )?;
        let conflicts: i64 =
            conn.query_row("SELECT count(*) FROM sync_conflicts", [], |row| row.get(0))?;
        let waiting: i64 =
            conn.query_row("SELECT count(*) FROM sync_deferred", [], |row| row.get(0))?;
        let clamp = |value: i64| u32::try_from(value.max(0)).unwrap_or(u32::MAX);
        Ok(SyncCounts {
            pending: clamp(outbox + pending),
            conflicts: clamp(conflicts),
            waiting: clamp(waiting),
        })
    }

    /// Plus grand numéro du journal local, pour détecter une modification récente.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn outbox_watermark(&self) -> AppResult<i64> {
        let conn = connection(&self.pool)?;
        Ok(
            conn.query_row("SELECT coalesce(max(seq), 0) FROM sync_outbox", [], |row| {
                row.get(0)
            })?,
        )
    }

    /// Curseurs : dernier lot appliqué de chaque autre installation.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn cursors(&self) -> AppResult<BTreeMap<String, u64>> {
        let conn = connection(&self.pool)?;
        let mut statement = conn.prepare("SELECT installation_id, last_seq FROM sync_cursors")?;
        let cursors = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(cursors
            .into_iter()
            .map(|(installation, seq)| (installation, u64::try_from(seq).unwrap_or(0)))
            .collect())
    }

    /// Dernier lot publié par cet ordinateur.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn last_published_seq(&self) -> AppResult<u64> {
        let conn = connection(&self.pool)?;
        let next = next_seq(&conn)?;
        let pending: Option<i64> =
            conn.query_row("SELECT min(seq) FROM sync_pending", [], |row| row.get(0))?;
        let unconfirmed = pending.and_then(|seq| u64::try_from(seq).ok());
        Ok(unconfirmed.unwrap_or(next).saturating_sub(1))
    }

    /// Lot préparé mais pas encore confirmé, à renvoyer en priorité.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn pending_batch(&self) -> AppResult<Option<PreparedBatch>> {
        let conn = connection(&self.pool)?;
        read_pending(&conn)
    }

    /// Prépare le prochain lot depuis le journal, ou renvoie le lot encore en attente.
    ///
    /// Le lot est encodé puis **enregistré** avant tout envoi, dans la même transaction que
    /// la mise à jour des versions et la purge du journal : après une coupure, c'est
    /// exactement le même contenu, sous le même nom, qui est renvoyé. Une écriture dont le
    /// résultat distant est incertain se rejoue donc sans effet de bord.
    ///
    /// Un enregistrement en conflit n'est pas publié tant que l'utilisateur n'a pas tranché :
    /// sa version locale porterait sinon une version qui écraserait l'autre en silence.
    ///
    /// # Errors
    /// Erreur SQLite ou de sérialisation ; rien n'est modifié en cas d'échec.
    pub fn prepare_batch(
        &self,
        photos_dir: &Path,
        blob_namer: &dyn Fn(&[u8]) -> String,
        encode: &dyn Fn(&str, &ChangeBatch) -> AppResult<Vec<u8>>,
    ) -> AppResult<Option<PreparedBatch>> {
        let mut conn = connection(&self.pool)?;
        if let Some(pending) = read_pending(&conn)? {
            return Ok(Some(pending));
        }
        let tx = conn.transaction()?;
        let config: SyncConfig = get_state(&tx, CONFIG_KEY)?
            .map_or_else(|| Ok(SyncConfig::default()), |text| rows::json_parse(&text))?;
        let Some((space_id, installation_id)) = config.attached() else {
            return Ok(None);
        };
        let watermark: i64 =
            tx.query_row("SELECT coalesce(max(seq), 0) FROM sync_outbox", [], |row| {
                row.get(0)
            })?;
        if watermark == 0 {
            return Ok(None);
        }
        let seq = next_seq(&tx)?;
        let records = dirty_records(&tx, watermark)?;
        let blocked = conflicted_records(&tx)?;
        let mut changes = Vec::new();
        let mut blobs = Vec::new();
        let mut done = Vec::new();
        for (entity, id) in records {
            if blocked.contains(&(entity.clone(), id.clone())) {
                continue;
            }
            done.push((entity.clone(), id.clone()));
            let Some(table) = entities::table(&entity) else {
                continue;
            };
            if let Some(mut change) = publish_record(&tx, table, &id, installation_id, seq)? {
                if table.name == entities::PROFILE.name {
                    if let Some((attachment, upload)) =
                        profile_attachment(change.data.as_ref(), photos_dir, blob_namer)
                    {
                        change.attachments.push(attachment);
                        blobs.push(upload);
                    }
                }
                changes.push(change);
            }
        }
        for (entity, id) in &done {
            tx.execute(
                "DELETE FROM sync_outbox WHERE entity = ?1 AND record_id = ?2 AND seq <= ?3",
                params![entity, id, watermark],
            )?;
        }
        if changes.is_empty() {
            tx.commit()?;
            return Ok(None);
        }
        let batch = ChangeBatch {
            format: FORMAT_VERSION,
            space_id: space_id.to_owned(),
            installation_id: installation_id.to_owned(),
            seq,
            schema_version: LATEST_SCHEMA_VERSION,
            created_at: now_iso(),
            changes,
        };
        let name = protocol::batch_name(installation_id, seq);
        let payload = encode(&name, &batch)?;
        let count = u32::try_from(batch.changes.len()).unwrap_or(u32::MAX);
        tx.execute(
            "INSERT INTO sync_pending (seq, object_name, payload, blobs, records, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                i64::try_from(seq).unwrap_or(i64::MAX),
                name,
                payload,
                rows::json_text(&blobs)?,
                count,
                now_iso()
            ],
        )?;
        set_state(&tx, NEXT_SEQ_KEY, &(seq + 1).to_string())?;
        tx.commit()?;
        Ok(Some(PreparedBatch {
            seq,
            name,
            payload,
            blobs,
            records: count,
        }))
    }

    /// Le fournisseur a confirmé l'écriture du lot.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn confirm_batch(&self, seq: u64) -> AppResult<()> {
        let conn = connection(&self.pool)?;
        conn.execute(
            "DELETE FROM sync_pending WHERE seq = ?1",
            [i64::try_from(seq).unwrap_or(i64::MAX)],
        )?;
        let since: u64 = get_state(&conn, SINCE_SNAPSHOT_KEY)?
            .and_then(|text| text.parse().ok())
            .unwrap_or(0);
        set_state(&conn, SINCE_SNAPSHOT_KEY, &(since + 1).to_string())?;
        Ok(())
    }

    /// Lots publiés depuis le dernier instantané.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn batches_since_snapshot(&self) -> AppResult<u64> {
        let conn = connection(&self.pool)?;
        Ok(get_state(&conn, SINCE_SNAPSHOT_KEY)?
            .and_then(|text| text.parse().ok())
            .unwrap_or(0))
    }

    /// Instantanés publiés par cet ordinateur.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn own_snapshots(&self) -> AppResult<Vec<OwnSnapshot>> {
        let conn = connection(&self.pool)?;
        get_state(&conn, OWN_SNAPSHOTS_KEY)?
            .map_or_else(|| Ok(Vec::new()), |text| rows::json_parse(&text))
    }

    /// Retient un instantané publié et remet le compteur à zéro.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn record_own_snapshot(&self, snapshots: &[OwnSnapshot]) -> AppResult<()> {
        let conn = connection(&self.pool)?;
        set_state(&conn, OWN_SNAPSHOTS_KEY, &rows::json_text(&snapshots)?)?;
        set_state(&conn, SINCE_SNAPSHOT_KEY, "0")
    }

    /// Instantané de l'état publié : contenus de référence (jamais les modifications locales
    /// non publiées) et marqueurs de suppression plus récents que `tombstones_since`.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn snapshot(
        &self,
        seq: u64,
        tombstones_since: &str,
        photos_dir: &Path,
        blob_namer: &dyn Fn(&[u8]) -> String,
    ) -> AppResult<Option<Snapshot>> {
        let conn = connection(&self.pool)?;
        let config: SyncConfig = get_state(&conn, CONFIG_KEY)?
            .map_or_else(|| Ok(SyncConfig::default()), |text| rows::json_parse(&text))?;
        let Some((space_id, installation_id)) = config.attached() else {
            return Ok(None);
        };
        let mut covered = VersionVector::new();
        for (installation, last) in self.cursors()? {
            covered.observe(&installation, last);
        }
        covered.observe(installation_id, self.last_published_seq()?);
        let mut statement = conn.prepare(
            "SELECT entity, record_id FROM sync_records
             WHERE deleted = 0 OR updated_at >= ?1 ORDER BY entity, record_id",
        )?;
        let keys = statement
            .query_map([tombstones_since], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut records = Vec::new();
        for (entity, id) in keys {
            let Some(meta) = rows::read_meta(&conn, &entity, &id)? else {
                continue;
            };
            if !meta.version.is_covered_by(&covered) {
                // Version locale pas encore publiée : l'instantané ne la porte pas.
                continue;
            }
            let data = if meta.deleted {
                None
            } else {
                meta.base.clone()
            };
            if !meta.deleted && data.is_none() {
                continue;
            }
            let mut change = RecordChange {
                entity: entity.clone(),
                id,
                version: meta.version,
                columns: meta.columns,
                data,
                attachments: Vec::new(),
            };
            if entity == entities::PROFILE.name {
                if let Some((attachment, _)) =
                    profile_attachment(change.data.as_ref(), photos_dir, blob_namer)
                {
                    change.attachments.push(attachment);
                }
            }
            records.push(change);
        }
        Ok(Some(Snapshot {
            format: FORMAT_VERSION,
            space_id: space_id.to_owned(),
            installation_id: installation_id.to_owned(),
            seq,
            schema_version: LATEST_SCHEMA_VERSION,
            created_at: now_iso(),
            covered,
            records,
        }))
    }

    /// Oublie les marqueurs de suppression plus anciens que la rétention.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn purge_tombstones(&self, before: &str) -> AppResult<usize> {
        let conn = connection(&self.pool)?;
        Ok(conn.execute(
            "DELETE FROM sync_records WHERE deleted = 1 AND updated_at < ?1",
            [before],
        )?)
    }

    /// Nom de la photo référencée par le profil local, s'il y en a une.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn profile_photo(&self) -> AppResult<Option<String>> {
        let conn = connection(&self.pool)?;
        let data: Option<String> = conn
            .query_row("SELECT data FROM profile WHERE id = 1", [], |row| {
                row.get(0)
            })
            .optional()?;
        Ok(data.and_then(|text| photo_of(&text)))
    }
}

fn next_seq(conn: &Connection) -> AppResult<u64> {
    Ok(get_state(conn, NEXT_SEQ_KEY)?
        .and_then(|text| text.parse().ok())
        .unwrap_or(1)
        .max(1))
}

fn read_pending(conn: &Connection) -> AppResult<Option<PreparedBatch>> {
    let raw = conn
        .query_row(
            "SELECT seq, object_name, payload, blobs, records FROM sync_pending
             ORDER BY seq LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        )
        .optional()?;
    raw.map(|(seq, name, payload, blobs, records)| {
        Ok(PreparedBatch {
            seq: u64::try_from(seq).unwrap_or(0),
            name,
            payload,
            blobs: rows::json_parse(&blobs)?,
            records: u32::try_from(records).unwrap_or(0),
        })
    })
    .transpose()
}

/// Enregistrements du journal jusqu'au numéro donné, parents d'abord.
fn dirty_records(tx: &Transaction<'_>, watermark: i64) -> AppResult<Vec<(String, String)>> {
    let mut statement = tx.prepare(
        "SELECT entity, record_id FROM sync_outbox WHERE seq <= ?1
         GROUP BY entity, record_id ORDER BY min(seq)",
    )?;
    let mut records = statement
        .query_map([watermark], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    records.sort_by_key(|(entity, _)| entities::rank(entity));
    Ok(records)
}

fn conflicted_records(conn: &Connection) -> AppResult<BTreeSet<(String, String)>> {
    let mut statement = conn.prepare("SELECT entity, record_id FROM sync_conflicts")?;
    let records = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<BTreeSet<_>, _>>()?;
    Ok(records)
}

/// Inscrit toutes les lignes synchronisées au journal.
pub fn enqueue_all(conn: &Connection) -> AppResult<()> {
    for table in SYNCED_TABLES {
        if table.name == entities::PROFILE.name {
            conn.execute(
                "INSERT INTO sync_outbox (entity, record_id)
                 SELECT 'profile', CAST(id AS TEXT) FROM profile WHERE data NOT IN ('{}', '')",
                [],
            )?;
            continue;
        }
        conn.execute(
            &format!(
                "INSERT INTO sync_outbox (entity, record_id) SELECT ?1, {} FROM {}",
                table.key, table.name
            ),
            [table.name],
        )?;
    }
    Ok(())
}

/// Version à publier pour un enregistrement modifié, et ses métadonnées mises à jour.
fn publish_record(
    tx: &Transaction<'_>,
    table: &SyncedTable,
    id: &str,
    installation_id: &str,
    seq: u64,
) -> AppResult<Option<RecordChange>> {
    let row = rows::read_row(tx, table, id)?;
    let meta = rows::read_meta(tx, table.name, id)?;
    let own = VersionVector::single(installation_id, seq);
    match (row, meta) {
        (None, None) => Ok(None),
        (None, Some(meta)) => {
            let version = meta.version.merged(&own);
            let tombstone = RecordMeta {
                version: version.clone(),
                columns: BTreeMap::new(),
                base: meta.base,
                forced: BTreeSet::new(),
                deleted: true,
            };
            rows::write_meta(tx, table.name, id, &tombstone)?;
            Ok(Some(RecordChange {
                entity: table.name.to_owned(),
                id: id.to_owned(),
                version,
                columns: BTreeMap::new(),
                data: None,
                attachments: Vec::new(),
            }))
        }
        (Some(row), meta) => {
            let pending = pending_columns(table, &row, meta.as_ref());
            let resurrected = meta.as_ref().is_some_and(|meta| meta.deleted);
            if pending.is_empty() && !resurrected && meta.is_some() {
                return Ok(None);
            }
            let previous = meta.unwrap_or_default();
            let version = previous.version.merged(&own);
            let mut columns = previous.columns;
            for column in &pending {
                columns.insert(column.clone(), version.clone());
            }
            let updated = RecordMeta {
                version: version.clone(),
                columns: columns.clone(),
                base: Some(row.clone()),
                forced: BTreeSet::new(),
                deleted: false,
            };
            rows::write_meta(tx, table.name, id, &updated)?;
            Ok(Some(RecordChange {
                entity: table.name.to_owned(),
                id: id.to_owned(),
                version,
                columns,
                data: Some(row),
                attachments: Vec::new(),
            }))
        }
    }
}

fn photo_of(profile_data: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(profile_data).ok()?;
    let name = value.get("photo")?.as_str()?;
    protocol::is_valid_photo_file(name).then(|| name.to_owned())
}

/// Fichier joint de la photo référencée par un contenu de profil.
fn profile_attachment(
    data: Option<&Row>,
    photos_dir: &Path,
    blob_namer: &dyn Fn(&[u8]) -> String,
) -> Option<(Attachment, BlobUpload)> {
    let text = data?.get("data")?.as_str()?;
    let file = photo_of(text)?;
    let bytes = match std::fs::read(photos_dir.join(&file)) {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::warn!(%error, "photo du profil absente : non jointe au lot");
            return None;
        }
    };
    let blob = blob_namer(&bytes);
    let sha256 = hex(&sha2_digest(&bytes));
    Some((
        Attachment {
            file: file.clone(),
            blob: blob.clone(),
            sha256,
            size: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        },
        BlobUpload { blob, file },
    ))
}

#[must_use]
pub fn sha2_digest(bytes: &[u8]) -> Vec<u8> {
    use sha2::Digest;
    sha2::Sha256::digest(bytes).to_vec()
}

#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::{open_pool, run_local_migrations};

    fn store() -> SqliteSyncStore {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        SqliteSyncStore::new(pool)
    }

    /// Une migration qui ajoute une colonne à une table synchronisée sans l'ajouter à la
    /// cartographie la laisserait hors de la synchronisation, sans erreur visible.
    #[test]
    fn la_cartographie_couvre_exactement_les_colonnes_du_schema() {
        let store = store();
        let conn = store.pool().get().unwrap();
        for table in SYNCED_TABLES {
            let mut statement = conn
                .prepare(&format!(
                    "SELECT name FROM pragma_table_info('{}')",
                    table.name
                ))
                .unwrap();
            let mut actual: Vec<String> = statement
                .query_map([], |row| row.get(0))
                .unwrap()
                .map(Result::unwrap)
                .collect();
            let mut declared: Vec<String> = table.columns.iter().map(|c| (*c).to_owned()).collect();
            actual.sort();
            declared.sort();
            assert_eq!(declared, actual, "colonnes de {}", table.name);
        }
    }

    #[test]
    fn chaque_table_synchronisee_journalise_ses_trois_operations() {
        let store = store();
        let conn = store.pool().get().unwrap();
        for table in SYNCED_TABLES {
            for operation in ["insert", "update", "delete"] {
                let name = format!("sync_{}_{operation}", table.name);
                let present: i64 = conn
                    .query_row(
                        "SELECT count(*) FROM sqlite_master WHERE type = 'trigger' AND name = ?1",
                        [&name],
                        |row| row.get(0),
                    )
                    .unwrap();
                assert_eq!(present, 1, "{name} absent");
            }
        }
    }

    #[test]
    fn une_ancienne_base_migree_garde_ses_donnees_sans_rien_journaliser() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("ancienne.sqlite");
        {
            let conn = rusqlite::Connection::open(&path).unwrap();
            conn.execute_batch(include_str!("../../../../migrations/init_schema.sql"))
                .unwrap();
            for migration in [
                include_str!("../../../../migrations/002_cover_letter_layout.sql"),
                include_str!("../../../../migrations/003_applications_reference_channel.sql"),
                include_str!("../../../../migrations/004_follow_up_done.sql"),
                include_str!("../../../../migrations/005_application_reference_counter.sql"),
                include_str!("../../../../migrations/006_saved_views.sql"),
                include_str!("../../../../migrations/007_relation_notes.sql"),
                include_str!("../../../../migrations/008_document_versions.sql"),
            ] {
                conn.execute_batch(migration).unwrap();
            }
            conn.execute_batch(
                "INSERT INTO companies (id, name, created_at, updated_at) VALUES ('c1', 'Acme', 't', 't');
                 PRAGMA user_version = 8;",
            )
            .unwrap();
        }
        let pool = open_pool(Some(&path)).unwrap();
        run_local_migrations(&pool).unwrap();
        let store = SqliteSyncStore::new(pool);
        let conn = store.pool().get().unwrap();
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, LATEST_SCHEMA_VERSION);
        let companies: i64 = conn
            .query_row("SELECT count(*) FROM companies", [], |row| row.get(0))
            .unwrap();
        assert_eq!(companies, 1);
        conn.execute("UPDATE companies SET name = 'Acme SA'", [])
            .unwrap();
        let outbox: i64 = conn
            .query_row("SELECT count(*) FROM sync_outbox", [], |row| row.get(0))
            .unwrap();
        assert_eq!(outbox, 0, "mode local : rien n'est journalisé");
        assert_eq!(store.config().unwrap().provider, SyncProviderKind::Local);
    }

    #[test]
    fn la_journalisation_est_atomique_avec_l_ecriture_metier() {
        let store = store();
        store
            .attach(
                &SyncConfig {
                    provider: SyncProviderKind::Folder,
                    folder: None,
                    space_id: Some("22222222-2222-4222-8222-222222222222".into()),
                    installation_id: Some("11111111-1111-4111-8111-111111111111".into()),
                    auto_sync: true,
                },
                false,
            )
            .unwrap();
        let mut conn = store.pool().get().unwrap();
        {
            let tx = conn.transaction().unwrap();
            tx.execute(
                "INSERT INTO companies (id, name, created_at, updated_at) VALUES ('c1', 'Acme', 't', 't')",
                [],
            )
            .unwrap();
            // Annulée : ni la ligne ni son entrée de journal ne subsistent.
            tx.rollback().unwrap();
        }
        let outbox: i64 = conn
            .query_row("SELECT count(*) FROM sync_outbox", [], |row| row.get(0))
            .unwrap();
        assert_eq!(outbox, 0);
        conn.execute(
            "INSERT INTO companies (id, name, created_at, updated_at) VALUES ('c1', 'Acme', 't', 't')",
            [],
        )
        .unwrap();
        let outbox: i64 = conn
            .query_row("SELECT count(*) FROM sync_outbox", [], |row| row.get(0))
            .unwrap();
        assert_eq!(outbox, 1);
    }

    #[test]
    fn un_instantane_se_prend_pendant_des_ecritures_concurrentes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("active.sqlite");
        let pool = open_pool(Some(&path)).unwrap();
        run_local_migrations(&pool).unwrap();
        let writer_pool = pool.clone();
        let writer = std::thread::spawn(move || {
            for index in 0..300 {
                writer_pool
                    .get()
                    .unwrap()
                    .execute(
                        "INSERT INTO companies (id, name, created_at, updated_at) VALUES (?1, 'x', 't', 't')",
                        [format!("c{index}")],
                    )
                    .unwrap();
            }
        });
        let copy = directory.path().join("instantane.sqlite");
        crate::core::backup::export(&pool, &copy).unwrap();
        writer.join().unwrap();
        crate::core::backup::validate(&copy).unwrap();
        let snapshot = rusqlite::Connection::open(&copy).unwrap();
        let integrity: String = snapshot
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        assert_eq!(integrity, "ok");
    }
}
