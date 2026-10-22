//! Moteur de synchronisation, indépendant du fournisseur (`docs/SYNC.md`, « Cycle »).
//!
//! Un cycle :
//!
//! 1. renvoie le lot préparé et non confirmé, s'il y en a un (coupure précédente) ;
//! 2. applique les lots des autres installations, dans l'ordre, sans jamais sauter un trou ;
//! 3. réessaie les changements en attente d'un parent ;
//! 4. publie un nouveau lot si des modifications locales attendent ;
//! 5. publie de temps à autre un instantané et fait le ménage de ses anciens lots.
//!
//! Le moteur ne parle qu'au contrat [`RemoteStore`] : aucune API Google ou Microsoft ici.

use crate::core::database::LATEST_SCHEMA_VERSION;
use crate::core::errors::{AppError, AppResult};
use crate::features::sync::domain::protocol::{
    self, ChangeBatch, ObjectKind, RecordChange, Snapshot, SpaceManifest, MANIFEST_NAME,
};
use crate::features::sync::domain::remote::{PutOutcome, RemoteError, RemoteStore};
use crate::features::sync::infrastructure::apply::{ApplyReport, Unit};
use crate::features::sync::infrastructure::crypto::{CryptoError, SpaceCipher};
use crate::features::sync::infrastructure::store::{
    hex, sha2_digest, OwnSnapshot, PreparedBatch, SqliteSyncStore, SyncConfig,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Lots publiés entre deux instantanés.
pub const SNAPSHOT_EVERY: u64 = 50;
/// Ancienneté d'un instantané au-delà de laquelle les lots qu'il couvre sont supprimés.
pub const BATCH_RETENTION_DAYS: i64 = 90;
/// Rétention des marqueurs de suppression : doit dépasser celle des lots, pour qu'un
/// ordinateur resté hors ligne moins longtemps reçoive toujours les suppressions.
pub const TOMBSTONE_RETENTION_DAYS: i64 = 180;
/// Lots publiés au plus par cycle : borne le travail d'un cycle sans jamais rien perdre.
const MAX_BATCHES_PER_CYCLE: usize = 5;

/// Échec d'un cycle, classé selon la réaction attendue.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SyncFailure {
    #[error("{0}")]
    Remote(RemoteError),
    /// Objet distant illisible, altéré ou incohérent.
    #[error("intégrité : {0}")]
    Integrity(String),
    /// Espace écrit par une version plus récente de Candilog.
    #[error("incompatible : {0}")]
    Incompatible(String),
    /// Erreur locale (base, disque).
    #[error("local : {0}")]
    Local(String),
}

impl From<RemoteError> for SyncFailure {
    fn from(error: RemoteError) -> Self {
        Self::Remote(error)
    }
}

impl From<AppError> for SyncFailure {
    fn from(error: AppError) -> Self {
        tracing::error!(%error, "erreur locale de synchronisation");
        Self::Local(error.user_message())
    }
}

/// Connexion à un espace : fournisseur, clé et identité de l'installation.
#[derive(Clone)]
pub struct Session {
    pub remote: Arc<dyn RemoteStore>,
    pub cipher: Arc<SpaceCipher>,
    pub space_id: String,
    pub installation_id: String,
}

/// Bilan d'un cycle.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CycleReport {
    pub received: u32,
    pub published: u32,
    pub conflicts: u32,
    /// Ordinateurs ayant publié dans l'espace.
    pub computers: u32,
    /// Premier échec rencontré ; le cycle continue autant que possible malgré lui.
    pub failure: Option<SyncFailure>,
    /// L'identité de l'installation a dû être renouvelée (base copiée d'un autre poste).
    pub renewed_installation: Option<String>,
}

impl CycleReport {
    fn fail(&mut self, failure: SyncFailure) {
        if self.failure.is_none() {
            self.failure = Some(failure);
        }
    }
}

/// Moteur de synchronisation d'une base locale.
#[derive(Clone)]
pub struct SyncEngine {
    store: SqliteSyncStore,
    photos_dir: PathBuf,
}

async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> AppResult<T> + Send + 'static,
) -> Result<T, SyncFailure> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|_| SyncFailure::Local("traitement interrompu".into()))?
        .map_err(SyncFailure::from)
}

fn integrity(error: CryptoError, name: &str) -> SyncFailure {
    tracing::warn!(%error, object = name, "objet distant illisible");
    SyncFailure::Integrity(format!(
        "un fichier de l'espace est illisible ou incomplet ({name})"
    ))
}

fn invalid(error: &protocol::InvalidObject) -> SyncFailure {
    if error.0.contains("plus récente") {
        SyncFailure::Incompatible(error.0.clone())
    } else {
        SyncFailure::Integrity(error.0.clone())
    }
}

/// Lit le manifeste d'un espace, `None` si l'emplacement est vide.
///
/// # Errors
/// Erreur du fournisseur, ou manifeste invalide.
pub async fn read_manifest(remote: &dyn RemoteStore) -> Result<Option<SpaceManifest>, SyncFailure> {
    let Some(bytes) = remote.get(MANIFEST_NAME).await? else {
        return Ok(None);
    };
    let manifest: SpaceManifest = serde_json::from_slice(&bytes)
        .map_err(|_| SyncFailure::Integrity("manifeste de l'espace illisible".into()))?;
    protocol::validate_manifest(&manifest).map_err(|error| invalid(&error))?;
    Ok(Some(manifest))
}

/// Publie le manifeste d'un nouvel espace. Un manifeste déjà présent n'est jamais remplacé.
///
/// # Errors
/// `Integrity` si un autre espace occupe déjà l'emplacement.
pub async fn publish_manifest(
    remote: &dyn RemoteStore,
    manifest: &SpaceManifest,
) -> Result<(), SyncFailure> {
    let bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|error| SyncFailure::Local(error.to_string()))?;
    match remote.put_new(MANIFEST_NAME, &bytes).await? {
        PutOutcome::Created => Ok(()),
        PutOutcome::AlreadyExists => Err(SyncFailure::Integrity(
            "un espace Candilog existe déjà à cet emplacement".into(),
        )),
    }
}

impl SyncEngine {
    #[must_use]
    pub const fn new(store: SqliteSyncStore, photos_dir: PathBuf) -> Self {
        Self { store, photos_dir }
    }

    #[must_use]
    pub const fn store(&self) -> &SqliteSyncStore {
        &self.store
    }

    #[must_use]
    pub fn photos_dir(&self) -> &Path {
        &self.photos_dir
    }

    /// Un cycle complet. Ne panique jamais et ne s'arrête pas au premier échec : une
    /// installation illisible n'empêche ni de lire les autres ni de publier.
    pub async fn cycle(&self, session: &Session) -> CycleReport {
        let mut report = CycleReport::default();
        if let Err(failure) = self.push(session, &mut report).await {
            report.fail(failure);
            // Sans le lot en attente confirmé, publier davantage n'aurait pas de sens ;
            // recevoir reste utile.
        }
        if let Err(failure) = self.pull(session, &mut report).await {
            report.fail(failure);
        }
        let store = self.store.clone();
        match blocking(move || store.retry_deferred()).await {
            Ok(applied) => self.after_apply(&applied, &mut report).await,
            Err(failure) => report.fail(failure),
        }
        if report.failure.is_none() {
            if let Err(failure) = self.push(session, &mut report).await {
                report.fail(failure);
            }
        }
        if report.failure.is_none() {
            if let Err(failure) = self.maintenance(session).await {
                // Le ménage est un confort : son échec ne doit pas masquer un cycle réussi.
                tracing::warn!(%failure, "maintenance de l'espace reportée");
            }
        }
        report
    }

    async fn after_apply(&self, applied: &ApplyReport, report: &mut CycleReport) {
        report.received += applied.applied;
        report.conflicts += applied.conflicts;
        if applied.profile_changed {
            let store = self.store.clone();
            let photos = self.photos_dir.clone();
            if let Err(failure) = blocking(move || {
                remove_orphan_photos(&photos, store.profile_photo()?.as_deref());
                Ok(())
            })
            .await
            {
                tracing::warn!(%failure, "ménage des photos reporté");
            }
        }
    }

    /// Publie les modifications locales : lot en attente d'abord, puis nouveaux lots.
    ///
    /// # Errors
    /// Erreur du fournisseur ou locale ; le lot reste en attente et sera renvoyé.
    pub async fn push(
        &self,
        session: &Session,
        report: &mut CycleReport,
    ) -> Result<(), SyncFailure> {
        for _ in 0..MAX_BATCHES_PER_CYCLE {
            let store = self.store.clone();
            let photos = self.photos_dir.clone();
            let cipher = Arc::clone(&session.cipher);
            let prepared = blocking(move || {
                let namer = |bytes: &[u8]| cipher.blob_id(bytes);
                let encode = |name: &str, batch: &ChangeBatch| cipher.encode_json(name, batch);
                store.prepare_batch(&photos, &namer, &encode)
            })
            .await?;
            let Some(batch) = prepared else {
                return Ok(());
            };
            if !batch.name.contains(&session.installation_id) {
                // Lot préparé sous une identité remplacée depuis : réinscrit au prochain lot.
                let store = self.store.clone();
                let id = session.installation_id.clone();
                blocking(move || store.renew_installation(&id)).await?;
                continue;
            }
            self.upload_blobs(session, &batch).await?;
            match session.remote.put_new(&batch.name, &batch.payload).await? {
                PutOutcome::Created => {}
                PutOutcome::AlreadyExists => {
                    // Résultat incertain d'un envoi précédent : le même contenu prouve que
                    // l'écriture avait abouti.
                    let existing = session.remote.get(&batch.name).await?;
                    if existing.as_deref() != Some(batch.payload.as_slice()) {
                        let renewed = uuid::Uuid::new_v4().to_string();
                        tracing::warn!(
                            "identité d'installation déjà utilisée dans l'espace : renouvelée"
                        );
                        let store = self.store.clone();
                        let id = renewed.clone();
                        blocking(move || store.renew_installation(&id)).await?;
                        report.renewed_installation = Some(renewed);
                        return Err(SyncFailure::Integrity(
                            "cette base a été copiée depuis un autre ordinateur : une nouvelle \
                             identité lui a été attribuée"
                                .into(),
                        ));
                    }
                }
            }
            let store = self.store.clone();
            let seq = batch.seq;
            blocking(move || store.confirm_batch(seq)).await?;
            report.published += batch.records;
        }
        Ok(())
    }

    async fn upload_blobs(
        &self,
        session: &Session,
        batch: &PreparedBatch,
    ) -> Result<(), SyncFailure> {
        for upload in &batch.blobs {
            let path = self.photos_dir.join(&upload.file);
            let content = match tokio::fs::read(&path).await {
                Ok(content) => content,
                Err(error) => {
                    tracing::warn!(%error, "fichier joint disparu avant envoi");
                    continue;
                }
            };
            let name = protocol::blob_name(&upload.blob);
            let sealed = session.cipher.seal(&name, &content)?;
            session.remote.put_new(&name, &sealed).await?;
        }
        Ok(())
    }

    /// Applique les lots des autres installations.
    ///
    /// # Errors
    /// Erreur du fournisseur ; les lots déjà appliqués restent acquis.
    pub async fn pull(
        &self,
        session: &Session,
        report: &mut CycleReport,
    ) -> Result<(), SyncFailure> {
        let listed = session.remote.list(protocol::CHANGES_PREFIX).await?;
        let mut by_installation: BTreeMap<String, BTreeSet<u64>> = BTreeMap::new();
        for object in listed {
            if let Some(ObjectKind::Batch {
                installation_id,
                seq,
            }) = protocol::parse_object_name(&object.name)
            {
                by_installation
                    .entry(installation_id)
                    .or_default()
                    .insert(seq);
            }
        }
        let mut computers: BTreeSet<&str> = by_installation.keys().map(String::as_str).collect();
        computers.insert(&session.installation_id);
        report.computers = u32::try_from(computers.len()).unwrap_or(u32::MAX);

        let store = self.store.clone();
        let mut cursors = blocking(move || store.cursors()).await?;
        let fresh = cursors.is_empty();
        let needs_snapshot = by_installation.iter().any(|(installation, seqs)| {
            installation != &session.installation_id
                && first_after(seqs, cursors.get(installation).copied().unwrap_or(0)).is_some_and(
                    |first| first > cursors.get(installation).copied().unwrap_or(0) + 1,
                )
        });
        if needs_snapshot || fresh {
            if let Some(applied) = self.catch_up(session, !fresh).await? {
                self.after_apply(&applied, report).await;
                let store = self.store.clone();
                cursors = blocking(move || store.cursors()).await?;
            }
        }

        for (installation, seqs) in &by_installation {
            if installation == &session.installation_id {
                continue;
            }
            let mut cursor = cursors.get(installation).copied().unwrap_or(0);
            let start = cursor;
            for seq in seqs.iter().copied().filter(|seq| *seq > start) {
                if seq != cursor + 1 {
                    // Trou : lot pas encore livré par l'outil de synchronisation, ou
                    // supprimé sans instantané qui le couvre. On n'avance jamais au-delà.
                    report.fail(SyncFailure::Integrity(
                        "des modifications d'un autre ordinateur ne sont pas encore disponibles"
                            .into(),
                    ));
                    break;
                }
                match self.apply_batch(session, installation, seq).await {
                    Ok(applied) => {
                        self.after_apply(&applied, report).await;
                        cursor = seq;
                    }
                    Err(failure @ SyncFailure::Remote(_)) => return Err(failure),
                    Err(failure) => {
                        report.fail(failure);
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    async fn apply_batch(
        &self,
        session: &Session,
        installation: &str,
        seq: u64,
    ) -> Result<ApplyReport, SyncFailure> {
        let name = protocol::batch_name(installation, seq);
        let Some(bytes) = session.remote.get(&name).await? else {
            return Err(SyncFailure::Integrity(format!(
                "lot annoncé mais absent ({name})"
            )));
        };
        let batch: ChangeBatch = session
            .cipher
            .decode_json(&name, &bytes)
            .map_err(|error| integrity(error, &name))?;
        protocol::validate_batch(&batch, &session.space_id, installation, seq)
            .map_err(|error| invalid(&error))?;
        if batch.schema_version > LATEST_SCHEMA_VERSION {
            return Err(SyncFailure::Incompatible(
                "un autre ordinateur utilise une version plus récente de Candilog".into(),
            ));
        }
        self.fetch_attachments(session, &batch.changes).await?;
        let store = self.store.clone();
        let installation = installation.to_owned();
        blocking(move || {
            store.apply_unit(Unit::Batch {
                installation_id: &installation,
                seq,
                changes: &batch.changes,
            })
        })
        .await
    }

    /// Instantané le plus complet de l'espace, s'il y en a un.
    async fn best_snapshot(&self, session: &Session) -> Result<Option<Snapshot>, SyncFailure> {
        let listed = session.remote.list(protocol::SNAPSHOTS_PREFIX).await?;
        let mut latest: BTreeMap<String, u64> = BTreeMap::new();
        for object in listed {
            if let Some(ObjectKind::Snapshot {
                installation_id,
                seq,
            }) = protocol::parse_object_name(&object.name)
            {
                let entry = latest.entry(installation_id).or_insert(0);
                *entry = (*entry).max(seq);
            }
        }
        let mut best: Option<Snapshot> = None;
        for (installation, seq) in latest {
            let name = protocol::snapshot_name(&installation, seq);
            let Some(bytes) = session.remote.get(&name).await? else {
                continue;
            };
            let snapshot: Snapshot = match session.cipher.decode_json(&name, &bytes) {
                Ok(snapshot) => snapshot,
                Err(error) => {
                    tracing::warn!(%error, "instantané illisible ignoré");
                    continue;
                }
            };
            if let Err(error) =
                protocol::validate_snapshot(&snapshot, &session.space_id, &installation, seq)
            {
                tracing::warn!(%error, "instantané invalide ignoré");
                continue;
            }
            let total =
                |snapshot: &Snapshot| snapshot.covered.entries().map(|(_, seq)| *seq).sum::<u64>();
            if best
                .as_ref()
                .is_none_or(|current| total(&snapshot) > total(current))
            {
                best = Some(snapshot);
            }
        }
        if let Some(snapshot) = &best {
            if snapshot.schema_version > LATEST_SCHEMA_VERSION {
                return Err(SyncFailure::Incompatible(
                    "un autre ordinateur utilise une version plus récente de Candilog".into(),
                ));
            }
        }
        Ok(best)
    }

    /// Applique le meilleur instantané : arrivée d'un nouvel ordinateur, ou retour d'un
    /// ordinateur dont les lots manquants ont été supprimés après la rétention.
    async fn catch_up(
        &self,
        session: &Session,
        prune_absent: bool,
    ) -> Result<Option<ApplyReport>, SyncFailure> {
        let Some(snapshot) = self.best_snapshot(session).await? else {
            return Ok(None);
        };
        self.fetch_attachments(session, &snapshot.records).await?;
        let store = self.store.clone();
        let applied = blocking(move || {
            store.apply_unit(Unit::Snapshot {
                covered: &snapshot.covered,
                records: &snapshot.records,
                prune_absent,
            })
        })
        .await?;
        Ok(Some(applied))
    }

    /// Télécharge tout l'espace (instantané puis lots) pour remplacer les données locales.
    ///
    /// # Errors
    /// Erreur du fournisseur, objet illisible ou incompatible : rien n'est appliqué.
    pub async fn replace_local_with_space(
        &self,
        session: &Session,
        config: &SyncConfig,
    ) -> Result<ApplyReport, SyncFailure> {
        let snapshot = self.best_snapshot(session).await?;
        let mut covered = snapshot
            .as_ref()
            .map(|s| s.covered.clone())
            .unwrap_or_default();
        let listed = session.remote.list(protocol::CHANGES_PREFIX).await?;
        let mut names: Vec<(String, u64)> = listed
            .iter()
            .filter_map(|object| match protocol::parse_object_name(&object.name) {
                Some(ObjectKind::Batch {
                    installation_id,
                    seq,
                }) => Some((installation_id, seq)),
                _ => None,
            })
            .filter(|(installation, seq)| *seq > covered.get(installation))
            .collect();
        names.sort();
        let mut units = Vec::new();
        for (installation, seq) in names {
            if seq != covered.get(&installation) + 1 {
                return Err(SyncFailure::Integrity(
                    "l'espace distant est incomplet : des modifications manquent".into(),
                ));
            }
            let name = protocol::batch_name(&installation, seq);
            let bytes = session
                .remote
                .get(&name)
                .await?
                .ok_or_else(|| SyncFailure::Integrity(format!("lot absent ({name})")))?;
            let batch: ChangeBatch = session
                .cipher
                .decode_json(&name, &bytes)
                .map_err(|error| integrity(error, &name))?;
            protocol::validate_batch(&batch, &session.space_id, &installation, seq)
                .map_err(|error| invalid(&error))?;
            if batch.schema_version > LATEST_SCHEMA_VERSION {
                return Err(SyncFailure::Incompatible(
                    "l'espace a été écrit par une version plus récente de Candilog".into(),
                ));
            }
            self.fetch_attachments(session, &batch.changes).await?;
            covered.observe(&installation, seq);
            units.push((installation, seq, batch.changes));
        }
        if let Some(snapshot) = &snapshot {
            self.fetch_attachments(session, &snapshot.records).await?;
        }
        let store = self.store.clone();
        let config = config.clone();
        let applied = blocking(move || {
            let snapshot_ref = snapshot
                .as_ref()
                .map(|s| (&s.covered, s.records.as_slice()));
            store.attach_replacing(&config, &units, snapshot_ref)
        })
        .await?;
        let photos = self.photos_dir.clone();
        let store = self.store.clone();
        blocking(move || {
            remove_orphan_photos(&photos, store.profile_photo()?.as_deref());
            Ok(())
        })
        .await?;
        Ok(applied)
    }

    /// Télécharge les fichiers joints absents localement, après vérification d'intégrité.
    /// Un fichier manquant n'empêche pas d'appliquer l'enregistrement : il est signalé.
    async fn fetch_attachments(
        &self,
        session: &Session,
        changes: &[RecordChange],
    ) -> Result<(), SyncFailure> {
        for change in changes {
            for attachment in &change.attachments {
                let target = self.photos_dir.join(&attachment.file);
                if tokio::fs::try_exists(&target).await.unwrap_or(false) {
                    continue;
                }
                let name = protocol::blob_name(&attachment.blob);
                let Some(sealed) = session.remote.get(&name).await? else {
                    tracing::warn!("fichier joint absent de l'espace");
                    continue;
                };
                let content = match session.cipher.open(&name, &sealed) {
                    Ok(content) => content,
                    Err(error) => {
                        tracing::warn!(%error, "fichier joint illisible");
                        continue;
                    }
                };
                if hex(&sha2_digest(&content)) != attachment.sha256
                    || u64::try_from(content.len()).ok() != Some(attachment.size)
                {
                    tracing::warn!("fichier joint dont l'empreinte ne correspond pas : ignoré");
                    continue;
                }
                let photos = self.photos_dir.clone();
                let file = attachment.file.clone();
                blocking(move || write_photo(&photos, &file, &content)).await?;
            }
        }
        Ok(())
    }

    /// Instantané périodique et ménage des lots anciens de cette installation.
    async fn maintenance(&self, session: &Session) -> Result<(), SyncFailure> {
        let store = self.store.clone();
        let (since, last_seq, snapshots, counts) = blocking(move || {
            Ok((
                store.batches_since_snapshot()?,
                store.last_published_seq()?,
                store.own_snapshots()?,
                store.counts()?,
            ))
        })
        .await?;
        let now = chrono::Utc::now();
        let tombstone_cutoff =
            (now - chrono::TimeDelta::days(TOMBSTONE_RETENTION_DAYS)).to_rfc3339();
        if since >= SNAPSHOT_EVERY
            && counts.pending == 0
            && last_seq > 0
            && snapshots
                .last()
                .is_none_or(|last| last.covered_self < last_seq)
        {
            self.write_snapshot(session, last_seq, snapshots, &tombstone_cutoff)
                .await?;
        }
        let store = self.store.clone();
        blocking(move || store.purge_tombstones(&tombstone_cutoff)).await?;
        Ok(())
    }

    /// Publie un instantané de l'état connu puis fait le ménage des lots qu'il rend inutiles.
    pub(crate) async fn write_snapshot(
        &self,
        session: &Session,
        last_seq: u64,
        snapshots: Vec<OwnSnapshot>,
        tombstone_cutoff: &str,
    ) -> Result<(), SyncFailure> {
        let store = self.store.clone();
        let photos = self.photos_dir.clone();
        let cipher = Arc::clone(&session.cipher);
        let cutoff = tombstone_cutoff.to_owned();
        let snapshot = blocking(move || {
            let namer = |bytes: &[u8]| cipher.blob_id(bytes);
            store.snapshot(last_seq, &cutoff, &photos, &namer)
        })
        .await?;
        let Some(snapshot) = snapshot else {
            return Ok(());
        };
        let name = protocol::snapshot_name(&session.installation_id, last_seq);
        let bytes = session.cipher.encode_json(&name, &snapshot)?;
        session.remote.put_new(&name, &bytes).await?;
        let mut kept = snapshots;
        kept.push(OwnSnapshot {
            seq: last_seq,
            covered_self: snapshot.covered.get(&session.installation_id),
            created_at: snapshot.created_at.clone(),
        });
        self.collect_garbage(session, &mut kept).await?;
        let store = self.store.clone();
        blocking(move || store.record_own_snapshot(&kept)).await
    }

    /// Supprime les lots de cette installation couverts par un de ses instantanés publié
    /// depuis plus de [`BATCH_RETENTION_DAYS`] jours, et les instantanés qu'il remplace.
    /// Une installation ne supprime jamais les fichiers d'une autre.
    async fn collect_garbage(
        &self,
        session: &Session,
        snapshots: &mut Vec<OwnSnapshot>,
    ) -> Result<(), SyncFailure> {
        let cutoff =
            (chrono::Utc::now() - chrono::TimeDelta::days(BATCH_RETENTION_DAYS)).to_rfc3339();
        let Some(anchor) = snapshots
            .iter()
            .filter(|snapshot| snapshot.created_at < cutoff)
            .max_by_key(|snapshot| snapshot.seq)
            .cloned()
        else {
            return Ok(());
        };
        let prefix = format!("{}{}/", protocol::CHANGES_PREFIX, session.installation_id);
        for object in session.remote.list(&prefix).await? {
            if let Some(ObjectKind::Batch { seq, .. }) = protocol::parse_object_name(&object.name) {
                if seq <= anchor.covered_self {
                    session.remote.delete(&object.name).await?;
                }
            }
        }
        for old in snapshots
            .iter()
            .filter(|snapshot| snapshot.seq < anchor.seq)
        {
            session
                .remote
                .delete(&protocol::snapshot_name(&session.installation_id, old.seq))
                .await?;
        }
        snapshots.retain(|snapshot| snapshot.seq >= anchor.seq);
        Ok(())
    }
}

fn first_after(seqs: &BTreeSet<u64>, cursor: u64) -> Option<u64> {
    seqs.range(cursor + 1..).next().copied()
}

fn write_photo(photos_dir: &Path, file: &str, content: &[u8]) -> AppResult<()> {
    if !protocol::is_valid_photo_file(file) {
        return Err(AppError::Validation("nom de photo refusé".into()));
    }
    std::fs::create_dir_all(photos_dir)
        .map_err(|error| AppError::Database(format!("dossier des photos : {error}")))?;
    let temporary = photos_dir.join(format!(".{file}.{}.tmp", uuid::Uuid::new_v4()));
    std::fs::write(&temporary, content)
        .map_err(|error| AppError::Database(format!("écriture de la photo : {error}")))?;
    crate::core::config::restreindre_fichier(&temporary);
    std::fs::rename(&temporary, photos_dir.join(file)).map_err(|error| {
        let _ = std::fs::remove_file(&temporary);
        AppError::Database(format!("écriture de la photo : {error}"))
    })
}

/// Supprime les photos de profil qui ne sont plus référencées après une synchronisation.
fn remove_orphan_photos(photos_dir: &Path, current: Option<&str>) {
    let Ok(entries) = std::fs::read_dir(photos_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if protocol::is_valid_photo_file(&name) && Some(name.as_str()) != current {
            if let Err(error) = std::fs::remove_file(entry.path()) {
                tracing::warn!(%error, "ancienne photo non supprimée");
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/engine/mod.rs"]
mod tests;
