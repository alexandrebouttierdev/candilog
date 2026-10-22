//! Service de synchronisation : configuration, mise en service, planification des cycles
//! et état affiché (`docs/SYNC.md`).

use crate::core::backup;
use crate::core::database::helpers::now_iso;
use crate::core::errors::{AppError, AppResult};
use crate::features::sync::application::engine::{
    publish_manifest, read_manifest, Session, SyncEngine, SyncFailure,
};
use crate::features::sync::domain::entities;
use crate::features::sync::domain::model::{
    column_label, ConflictChoice, ConflictField, ProviderAvailability, RemoteProbe, SetupChoice,
    SetupOutcome, SetupRequest, SyncConflict, SyncPhase, SyncProviderKind, SyncStatus,
};
use crate::features::sync::domain::protocol::Row;
use crate::features::sync::domain::remote::{RemoteError, RemoteStore};
use crate::features::sync::infrastructure::apply::StoredConflict;
use crate::features::sync::infrastructure::folder::FolderStore;
use crate::features::sync::infrastructure::oauth::{self, OAuthClient, OAuthError, OAuthSession};
use crate::features::sync::infrastructure::secrets::{space_entry, SyncSecrets};
use crate::features::sync::infrastructure::store::{SqliteSyncStore, SyncConfig};
use crate::features::sync::infrastructure::{google_drive, onedrive};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

/// Intervalle de surveillance du journal local.
const TICK: Duration = Duration::from_secs(5);
/// Regroupement : une modification attend ce délai de calme avant de partir.
const DEBOUNCE: Duration = Duration::from_secs(5);
/// Au-delà, une modification part même si l'utilisateur continue de saisir.
const MAX_DELAY: Duration = Duration::from_secs(60);
/// Relève périodique des changements des autres ordinateurs.
const POLL_CLOUD: Duration = Duration::from_secs(300);
const POLL_FOLDER: Duration = Duration::from_secs(120);
/// Délais croissants après des échecs passagers consécutifs.
const BACKOFF: [u64; 6] = [30, 60, 120, 300, 600, 900];

/// Fonction de notification de l'état, posée par la couche de présentation.
pub type StatusListener = Box<dyn Fn(&SyncStatus) + Send + Sync>;

/// Emplacement choisi, en attente du choix de scénario.
#[derive(Clone)]
struct Pending {
    provider: SyncProviderKind,
    folder: Option<PathBuf>,
    remote: Arc<dyn RemoteStore>,
    oauth: Option<Arc<OAuthSession>>,
}

/// Blocage qui interdit les cycles automatiques jusqu'à une action de l'utilisateur.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Blocked {
    phase: SyncPhase,
    message: String,
}

/// Ce qui empêche d'ouvrir une session : un blocage, ou un échec passager.
enum SessionError {
    Blocked(Blocked),
    Failure(SyncFailure),
}

#[derive(Default)]
struct Runtime {
    syncing: bool,
    restoring: bool,
    blocked: Option<Blocked>,
    /// Dernier échec passager : phase (hors ligne / erreur) et explication.
    transient: Option<(SyncPhase, String)>,
    failures: u32,
    next_attempt: Option<Instant>,
    next_attempt_at: Option<String>,
    last_attempt_at: Option<String>,
    succeeded_once: bool,
    last_cycle: Option<Instant>,
    watermark: i64,
    watermark_changed: Option<Instant>,
    computers: u32,
}

/// Service partagé par les commandes et la tâche de fond.
pub struct SyncService {
    engine: SyncEngine,
    exports_dir: PathBuf,
    secrets: Arc<dyn SyncSecrets>,
    runtime: Mutex<Runtime>,
    pending: Mutex<Option<Pending>>,
    connect_cancel: Mutex<Option<CancellationToken>>,
    oauth: Mutex<Option<Arc<OAuthSession>>>,
    listener: Mutex<Option<StatusListener>>,
    notify: Notify,
    shutdown: CancellationToken,
    cycle_lock: tokio::sync::Mutex<()>,
}

fn sync_error(message: impl Into<String>) -> AppError {
    AppError::Sync(message.into())
}

/// Message français d'un échec distant, et phase correspondante.
fn describe_remote(error: &RemoteError, provider: SyncProviderKind) -> (SyncPhase, String) {
    match error {
        RemoteError::Offline(_) => (
            SyncPhase::Offline,
            "Pas de connexion : vos modifications sont conservées et partiront au retour du réseau.".into(),
        ),
        RemoteError::RateLimited(_) => (
            SyncPhase::TemporaryError,
            format!("{} limite temporairement les échanges : nouvel essai automatique.", provider.label()),
        ),
        RemoteError::Unauthorized(_) => (
            SyncPhase::AuthRequired,
            format!("La connexion à {} a expiré ou a été révoquée : reconnectez-vous.", provider.label()),
        ),
        RemoteError::Unavailable(detail) => (
            SyncPhase::TemporaryError,
            if provider == SyncProviderKind::Folder {
                "Le dossier de synchronisation est inaccessible : vérifiez qu'il est monté et que vous pouvez y écrire.".into()
            } else {
                format!("{} refuse l'accès ({detail}).", provider.label())
            },
        ),
        RemoteError::Provider(_) => (
            SyncPhase::TemporaryError,
            format!("{} a renvoyé une erreur : nouvel essai automatique.", provider.label()),
        ),
    }
}

fn user_message_of(failure: &SyncFailure, provider: SyncProviderKind) -> String {
    match failure {
        SyncFailure::Remote(error) => describe_remote(error, provider).1,
        SyncFailure::Integrity(detail) => format!("Synchronisation incomplète : {detail}."),
        SyncFailure::Incompatible(_) => {
            "Un autre ordinateur utilise une version plus récente de Candilog : mettez à jour cette installation.".into()
        }
        SyncFailure::Local(detail) => detail.clone(),
    }
}

fn into_app_error(failure: &SyncFailure, provider: SyncProviderKind) -> AppError {
    sync_error(user_message_of(failure, provider))
}

fn oauth_message(error: &OAuthError, provider: SyncProviderKind) -> AppError {
    if *error == OAuthError::Cancelled {
        return AppError::Cancelled;
    }
    sync_error(match error {
        OAuthError::Denied => format!(
            "Vous n'avez pas autorisé Candilog à accéder à {}.",
            provider.label()
        ),
        OAuthError::Cancelled => "Connexion annulée.".into(),
        OAuthError::TimedOut => "La connexion n'a pas été terminée à temps : recommencez.".into(),
        OAuthError::InvalidCallback => {
            "La réponse de connexion ne correspond pas à la demande : recommencez depuis Candilog."
                .into()
        }
        OAuthError::Revoked => format!("{} a refusé la connexion : recommencez.", provider.label()),
        OAuthError::Offline => format!(
            "{} est injoignable : vérifiez votre connexion.",
            provider.label()
        ),
        OAuthError::Provider(_) => format!(
            "{} a renvoyé une erreur pendant la connexion.",
            provider.label()
        ),
    })
}

impl SyncService {
    #[must_use]
    pub fn new(
        store: SqliteSyncStore,
        photos_dir: PathBuf,
        exports_dir: PathBuf,
        secrets: Arc<dyn SyncSecrets>,
    ) -> Self {
        Self {
            engine: SyncEngine::new(store, photos_dir),
            exports_dir,
            secrets,
            runtime: Mutex::new(Runtime::default()),
            pending: Mutex::new(None),
            connect_cancel: Mutex::new(None),
            oauth: Mutex::new(None),
            listener: Mutex::new(None),
            notify: Notify::new(),
            shutdown: CancellationToken::new(),
            cycle_lock: tokio::sync::Mutex::new(()),
        }
    }

    fn store(&self) -> &SqliteSyncStore {
        self.engine.store()
    }

    fn runtime(&self) -> std::sync::MutexGuard<'_, Runtime> {
        // Un verrou empoisonné ne contient qu'un état d'affichage : on le reprend tel quel.
        self.runtime
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Pose la fonction appelée à chaque changement d'état.
    pub fn set_listener(&self, listener: StatusListener) {
        *self
            .listener
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(listener);
    }

    fn publish_status(&self) {
        match self.status() {
            Ok(status) => {
                if let Some(listener) = self
                    .listener
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .as_ref()
                {
                    listener(&status);
                }
            }
            Err(error) => tracing::warn!(%error, "état de synchronisation non calculé"),
        }
    }

    /// Fournisseurs proposés par cette version.
    #[must_use]
    pub fn availability() -> ProviderAvailability {
        ProviderAvailability {
            google_drive: google_drive::oauth_client().is_some(),
            one_drive: onedrive::oauth_client().is_some(),
        }
    }

    /// État courant.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn status(&self) -> AppResult<SyncStatus> {
        let config = self.store().config()?;
        if config.attached().is_none() {
            return Ok(SyncStatus::local());
        }
        let counts = self.store().counts()?;
        let missing_files = match self.store().profile_photo()? {
            Some(photo) if !self.engine.photos_dir().join(&photo).exists() => 1,
            _ => 0,
        };
        let runtime = self.runtime();
        let (phase, message) = if runtime.restoring {
            (
                SyncPhase::Restoring,
                Some("Restauration des données de l'espace en cours.".into()),
            )
        } else if runtime.syncing {
            (SyncPhase::Syncing, None)
        } else if let Some(blocked) = &runtime.blocked {
            (blocked.phase, Some(blocked.message.clone()))
        } else if !config.auto_sync {
            (
                SyncPhase::Disabled,
                Some("Synchronisation automatique suspendue : vos modifications sont conservées et partiront à la reprise.".into()),
            )
        } else if let Some((phase, message)) = &runtime.transient {
            (*phase, Some(message.clone()))
        } else if counts.conflicts > 0 {
            (
                SyncPhase::Conflict,
                Some(
                    "Des modifications faites sur deux ordinateurs attendent votre arbitrage."
                        .into(),
                ),
            )
        } else if counts.pending > 0 || !runtime.succeeded_once {
            (SyncPhase::Pending, None)
        } else {
            (SyncPhase::UpToDate, None)
        };
        Ok(SyncStatus {
            provider: config.provider,
            phase,
            auto_sync: config.auto_sync,
            location: Some(match (&config.provider, &config.folder) {
                (SyncProviderKind::Folder, Some(folder)) => folder.clone(),
                (provider, _) => provider.label().to_owned(),
            }),
            pending_changes: counts.pending,
            conflicts: counts.conflicts,
            waiting_changes: counts.waiting,
            missing_files,
            computers: runtime.computers,
            last_success_at: self.store().last_success()?,
            last_attempt_at: runtime.last_attempt_at.clone(),
            next_attempt_at: runtime.next_attempt_at.clone(),
            message,
        })
    }

    fn oauth_client(provider: SyncProviderKind) -> AppResult<OAuthClient> {
        let client = match provider {
            SyncProviderKind::GoogleDrive => google_drive::oauth_client(),
            SyncProviderKind::OneDrive => onedrive::oauth_client(),
            _ => None,
        };
        client.ok_or_else(|| {
            sync_error(format!(
                "{} n'est pas disponible dans cette version de Candilog : son application OAuth n'a pas été configurée à la compilation.",
                provider.label()
            ))
        })
    }

    fn oauth_session(&self, provider: SyncProviderKind) -> AppResult<Arc<OAuthSession>> {
        let mut cached = self
            .oauth
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(session) = cached.as_ref() {
            return Ok(Arc::clone(session));
        }
        let http = oauth::http_client().map_err(|error| sync_error(error.to_string()))?;
        let session = Arc::new(OAuthSession::new(
            Self::oauth_client(provider)?,
            http,
            Arc::clone(&self.secrets),
        ));
        *cached = Some(Arc::clone(&session));
        Ok(session)
    }

    fn remote_for(&self, config: &SyncConfig) -> AppResult<Arc<dyn RemoteStore>> {
        match config.provider {
            SyncProviderKind::Folder => {
                let folder = config
                    .folder
                    .as_deref()
                    .ok_or_else(|| sync_error("Dossier de synchronisation non défini."))?;
                Ok(Arc::new(FolderStore::existing(Path::new(folder)).map_err(
                    |error| sync_error(describe_remote(&error, SyncProviderKind::Folder).1),
                )?))
            }
            SyncProviderKind::GoogleDrive => Ok(Arc::new(google_drive::GoogleDriveStore::new(
                self.oauth_session(config.provider)?,
            ))),
            SyncProviderKind::OneDrive => Ok(Arc::new(onedrive::OneDriveStore::new(
                self.oauth_session(config.provider)?,
            ))),
            SyncProviderKind::Local => Err(sync_error("Aucun fournisseur choisi.")),
        }
    }

    /// Session de l'espace rattaché, ou ce qui l'empêche.
    fn session(&self, config: &SyncConfig) -> Result<Session, SessionError> {
        let (space_id, installation_id) = config.attached().ok_or_else(|| {
            SessionError::Blocked(Blocked {
                phase: SyncPhase::NotConfigured,
                message: String::new(),
            })
        })?;
        let remote = self.remote_for(config).map_err(|error| {
            // Dossier démonté, fournisseur indisponible : passager, réessayé avec délai.
            SessionError::Failure(SyncFailure::Remote(RemoteError::Unavailable(
                error.user_message(),
            )))
        })?;
        let session = Session {
            remote,
            space_id: space_id.to_owned(),
            installation_id: installation_id.to_owned(),
        };
        Ok(session)
    }

    async fn validate_session_space(&self, session: &Session) -> Result<(), SyncFailure> {
        let manifest = read_manifest(session.remote.as_ref()).await?;
        match manifest {
            Some(manifest) if manifest.space_id == session.space_id => Ok(()),
            Some(_) => Err(SyncFailure::Incompatible(
                "Cet emplacement contient un autre espace de synchronisation : désactivez puis configurez un nouvel espace.".into(),
            )),
            None => Err(SyncFailure::Incompatible(
                "Cet espace utilise un ancien format chiffré et n'a pas été modifié. Désactivez la synchronisation puis créez un nouvel espace non chiffré.".into(),
            )),
        }
    }

    async fn probe(
        &self,
        provider: SyncProviderKind,
        location: String,
        remote: &dyn RemoteStore,
    ) -> AppResult<RemoteProbe> {
        let manifest = read_manifest(remote)
            .await
            .map_err(|failure| into_app_error(&failure, provider))?;
        let legacy_space_exists = remote
            .get("space.json")
            .await
            .map_err(|error| into_app_error(&SyncFailure::Remote(error), provider))?
            .is_some();
        let store = self.store().clone();
        let local_has_data =
            crate::core::utils::blocking::execute(move || store.local_has_data()).await?;
        Ok(RemoteProbe {
            provider,
            location,
            space_exists: manifest.is_some(),
            legacy_space_exists,
            space_created_at: manifest.map(|manifest| manifest.created_at),
            local_has_data,
        })
    }

    /// Retient un dossier choisi par l'utilisateur, après avoir vérifié qu'on peut y écrire.
    ///
    /// # Errors
    /// `Sync` si le dossier est inaccessible, ou situé dans les données de Candilog.
    pub async fn choose_folder(&self, folder: PathBuf, data_dir: &Path) -> AppResult<RemoteProbe> {
        let canonical = std::fs::canonicalize(&folder)
            .map_err(|_| sync_error("Ce dossier est introuvable."))?;
        if let Ok(data) = std::fs::canonicalize(data_dir) {
            if canonical.starts_with(&data) || data.starts_with(&canonical) {
                return Err(sync_error(
                    "Choisissez un dossier extérieur aux données de Candilog : la base active ne doit jamais être synchronisée comme un fichier.",
                ));
            }
        }
        let store = FolderStore::open(&canonical)
            .map_err(|error| sync_error(describe_remote(&error, SyncProviderKind::Folder).1))?;
        let remote: Arc<dyn RemoteStore> = Arc::new(store);
        let location = canonical.display().to_string();
        let probe = self
            .probe(SyncProviderKind::Folder, location, remote.as_ref())
            .await?;
        *self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Pending {
            provider: SyncProviderKind::Folder,
            folder: Some(canonical),
            remote,
            oauth: None,
        });
        Ok(probe)
    }

    /// Connecte un compte Google Drive ou OneDrive : autorisation dans le navigateur, puis
    /// inspection de l'emplacement, sans encore rien échanger.
    ///
    /// # Errors
    /// `Sync` : refus, abandon, réseau, fournisseur non disponible dans cette version.
    pub async fn connect_cloud(
        &self,
        provider: SyncProviderKind,
        open_browser: &(dyn Fn(&str) -> Result<(), String> + Send + Sync),
    ) -> AppResult<RemoteProbe> {
        let client = Self::oauth_client(provider)?;
        let http = oauth::http_client().map_err(|error| sync_error(error.to_string()))?;
        let cancel = CancellationToken::new();
        *self
            .connect_cancel
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(cancel.clone());
        let tokens = oauth::authorize(&client, &http, open_browser, &cancel).await;
        *self
            .connect_cancel
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        let tokens = tokens.map_err(|error| oauth_message(&error, provider))?;
        let session = Arc::new(OAuthSession::new(client, http, Arc::clone(&self.secrets)));
        session
            .adopt(tokens)
            .await
            .map_err(|error| sync_error(describe_remote(&error, provider).1))?;
        let remote: Arc<dyn RemoteStore> = match provider {
            SyncProviderKind::GoogleDrive => {
                Arc::new(google_drive::GoogleDriveStore::new(Arc::clone(&session)))
            }
            _ => Arc::new(onedrive::OneDriveStore::new(Arc::clone(&session))),
        };
        let probe = self
            .probe(provider, provider.label().to_owned(), remote.as_ref())
            .await?;
        *self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Pending {
            provider,
            folder: None,
            remote,
            oauth: Some(session),
        });
        Ok(probe)
    }

    /// Abandonne une connexion en cours dans le navigateur.
    pub fn cancel_connect(&self) {
        if let Some(cancel) = self
            .connect_cancel
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            cancel.cancel();
        }
    }

    /// Met la synchronisation en service selon le scénario choisi explicitement.
    ///
    /// # Errors
    /// `Validation` si le scénario ne correspond pas à l'emplacement ; `Sync` pour un échec
    /// du fournisseur ; rien n'est rattaché en cas d'échec.
    pub async fn setup(&self, request: SetupRequest) -> AppResult<SetupOutcome> {
        let pending = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .ok_or_else(|| {
                AppError::Validation("Choisissez d'abord un emplacement de synchronisation.".into())
            })?;
        let provider = pending.provider;
        let manifest = read_manifest(pending.remote.as_ref())
            .await
            .map_err(|failure| into_app_error(&failure, provider))?;
        let installation_id = uuid::Uuid::new_v4().to_string();
        let mut outcome = SetupOutcome { backup_file: None };

        let space_id = match (request.choice, &manifest) {
            (SetupChoice::CreateSpace, Some(_)) => {
                return Err(AppError::Validation(
                    "Un espace Candilog existe déjà à cet emplacement : rejoignez-le plutôt que d'en créer un autre.".into(),
                ))
            }
            (SetupChoice::CreateSpace, None) => {
                let space_id = uuid::Uuid::new_v4().to_string();
                let new_manifest = crate::features::sync::domain::protocol::SpaceManifest {
                    format: crate::features::sync::domain::protocol::FORMAT_VERSION,
                    app: "candilog".into(),
                    space_id: space_id.clone(),
                    created_at: now_iso(),
                };
                publish_manifest(pending.remote.as_ref(), &new_manifest)
                    .await
                    .map_err(|failure| into_app_error(&failure, provider))?;
                space_id
            }
            (_, None) => {
                return Err(AppError::Validation(
                    "Aucun espace Candilog n'existe encore à cet emplacement : créez-le depuis l'ordinateur qui contient vos données.".into(),
                ))
            }
            (_, Some(manifest)) => {
                manifest.space_id.clone()
            }
        };

        let config = SyncConfig {
            provider,
            folder: pending
                .folder
                .as_ref()
                .map(|path| path.display().to_string()),
            space_id: Some(space_id.clone()),
            installation_id: Some(installation_id.clone()),
            auto_sync: true,
        };
        if let Some(session) = &pending.oauth {
            *self
                .oauth
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::clone(session));
        }

        let attached = match request.choice {
            SetupChoice::JoinReplaceLocal => {
                outcome.backup_file = Some(self.safety_backup().await?);
                let session = Session {
                    remote: Arc::clone(&pending.remote),
                    space_id: space_id.clone(),
                    installation_id: installation_id.clone(),
                };
                self.validate_session_space(&session)
                    .await
                    .map_err(|failure| into_app_error(&failure, provider))?;
                self.runtime().restoring = true;
                self.publish_status();
                let result = self
                    .engine
                    .replace_local_with_space(&session, &config)
                    .await;
                self.runtime().restoring = false;
                result
                    .map(|_| ())
                    .map_err(|failure| into_app_error(&failure, provider))
            }
            SetupChoice::CreateSpace | SetupChoice::JoinMerge => {
                let store = self.store().clone();
                let config = config.clone();
                crate::core::utils::blocking::execute(move || store.attach(&config, true)).await
            }
        };
        attached?;
        *self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        self.reset_runtime();
        self.trigger();
        Ok(outcome)
    }

    async fn safety_backup(&self) -> AppResult<String> {
        let pool = self.store().pool().clone();
        let directory = self.exports_dir.clone();
        crate::core::utils::blocking::execute(move || {
            std::fs::create_dir_all(&directory).map_err(|error| {
                AppError::Database(format!("dossier des sauvegardes : {error}"))
            })?;
            let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
            let path = directory.join(format!("candilog-avant-synchronisation-{stamp}.sqlite"));
            backup::export(&pool, &path)?;
            Ok(path.display().to_string())
        })
        .await
    }

    /// Remplace les données locales par celles de l'espace rattaché, après une sauvegarde.
    ///
    /// # Errors
    /// `Sync` si l'espace est inaccessible ; les données locales restent alors intactes.
    pub async fn restore_from_space(&self) -> AppResult<String> {
        let _guard = self.cycle_lock.lock().await;
        let config = self.store().config()?;
        let session = self.session(&config).map_err(|error| match error {
            SessionError::Blocked(blocked) => sync_error(blocked.message),
            SessionError::Failure(failure) => into_app_error(&failure, config.provider),
        })?;
        self.validate_session_space(&session)
            .await
            .map_err(|failure| into_app_error(&failure, config.provider))?;
        let backup_file = self.safety_backup().await?;
        self.runtime().restoring = true;
        self.publish_status();
        let result = self
            .engine
            .replace_local_with_space(&session, &config)
            .await;
        self.runtime().restoring = false;
        result.map_err(|failure| into_app_error(&failure, config.provider))?;
        self.reset_runtime();
        self.trigger();
        Ok(backup_file)
    }

    /// Active ou suspend la synchronisation automatique.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn set_auto(&self, enabled: bool) -> AppResult<()> {
        let mut config = self.store().config()?;
        config.auto_sync = enabled;
        self.store().save_config(&config)?;
        if enabled {
            self.trigger();
        } else {
            self.publish_status();
        }
        Ok(())
    }

    /// Détache l'ordinateur de son espace : les données locales et l'espace distant sont
    /// laissés intacts ; seuls l'état de synchronisation et les secrets locaux disparaissent.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub async fn disconnect(&self) -> AppResult<()> {
        let _guard = self.cycle_lock.lock().await;
        let config = self.store().config()?;
        let store = self.store().clone();
        crate::core::utils::blocking::execute(move || store.detach()).await?;
        self.forget_secrets(&config).await;
        self.reset_runtime();
        self.publish_status();
        Ok(())
    }

    async fn forget_secrets(&self, config: &SyncConfig) {
        if let Some(space_id) = &config.space_id {
            if let Err(error) = self.secrets.delete(&space_entry(space_id)) {
                tracing::warn!(%error, "clé de synchronisation non retirée du coffre");
            }
        }
        let session = self
            .oauth
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(session) = session {
            session.sign_out().await;
        } else if matches!(
            config.provider,
            SyncProviderKind::GoogleDrive | SyncProviderKind::OneDrive
        ) {
            if let Ok(client) = Self::oauth_client(config.provider) {
                if let Ok(http) = oauth::http_client() {
                    OAuthSession::new(client, http, Arc::clone(&self.secrets))
                        .sign_out()
                        .await;
                }
            }
        }
    }

    /// À appeler après une remise à zéro ou une restauration de sauvegarde, qui ont
    /// détaché la base : la synchronisation repasse en mode local.
    pub async fn after_local_replacement(&self, previous: &SyncConfig) {
        self.forget_secrets(previous).await;
        self.reset_runtime();
        self.publish_status();
    }

    /// Configuration courante (pour l'appeler avant une opération qui la détruit).
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn config(&self) -> AppResult<SyncConfig> {
        self.store().config()
    }

    fn reset_runtime(&self) {
        *self.runtime() = Runtime::default();
    }

    /// Demande un cycle immédiat.
    pub fn trigger(&self) {
        {
            let mut runtime = self.runtime();
            runtime.next_attempt = None;
            runtime.next_attempt_at = None;
            // Une action de l'utilisateur lève les blocages : on retente pour vérifier.
            runtime.blocked = None;
        }
        self.notify.notify_one();
        self.publish_status();
    }

    /// Conflits en attente, présentés.
    ///
    /// # Errors
    /// Erreur SQLite.
    pub fn conflicts(&self) -> AppResult<Vec<SyncConflict>> {
        Ok(self
            .store()
            .conflicts()?
            .into_iter()
            .map(present_conflict)
            .collect())
    }

    /// Applique une décision sur un conflit.
    ///
    /// # Errors
    /// `NotFound` ou `Validation`.
    pub fn resolve(&self, conflict_id: &str, choice: ConflictChoice) -> AppResult<()> {
        self.store().resolve_conflict(conflict_id, choice)?;
        self.trigger();
        Ok(())
    }

    /// Applique la même décision à tous les conflits.
    ///
    /// # Errors
    /// Première erreur rencontrée ; les conflits déjà résolus le restent.
    pub fn resolve_all(&self, choice: ConflictChoice) -> AppResult<u32> {
        let mut resolved = 0;
        for conflict in self.store().conflicts()? {
            self.store().resolve_conflict(&conflict.id, choice)?;
            resolved += 1;
        }
        self.trigger();
        Ok(resolved)
    }

    /// Arrête la tâche de fond (fermeture de l'application).
    pub fn shutdown(&self) {
        self.shutdown.cancel();
        self.cancel_connect();
    }

    /// Exécute un cycle maintenant, en attendant sa fin.
    pub async fn run_cycle(&self) {
        let _guard = self.cycle_lock.lock().await;
        let config = match self.store().config() {
            Ok(config) => config,
            Err(error) => {
                tracing::error!(%error, "configuration de synchronisation illisible");
                return;
            }
        };
        if config.attached().is_none() {
            return;
        }
        let session = match self.session(&config) {
            Ok(opened) => opened,
            Err(SessionError::Blocked(blocked)) => {
                self.runtime().blocked = Some(blocked);
                self.publish_status();
                return;
            }
            Err(SessionError::Failure(failure)) => {
                let mut runtime = self.runtime();
                runtime.last_attempt_at = Some(now_iso());
                self.schedule_retry(&mut runtime, &failure, config.provider);
                drop(runtime);
                self.publish_status();
                return;
            }
        };
        if let Err(failure) = self.validate_session_space(&session).await {
            match failure {
                SyncFailure::Incompatible(message) => {
                    self.runtime().blocked = Some(Blocked {
                        phase: SyncPhase::Incompatible,
                        message,
                    });
                }
                failure => {
                    let mut runtime = self.runtime();
                    self.schedule_retry(&mut runtime, &failure, config.provider);
                }
            }
            self.publish_status();
            return;
        }
        {
            let mut runtime = self.runtime();
            runtime.syncing = true;
            runtime.last_attempt_at = Some(now_iso());
        }
        self.publish_status();
        let report = tokio::select! {
            () = self.shutdown.cancelled() => return,
            report = self.engine.cycle(&session) => report,
        };
        let pending = self
            .store()
            .counts()
            .map(|counts| counts.pending)
            .unwrap_or(1);
        let mut runtime = self.runtime();
        runtime.syncing = false;
        runtime.last_cycle = Some(Instant::now());
        if report.computers > 0 {
            runtime.computers = report.computers;
        }
        match &report.failure {
            None => {
                runtime.failures = 0;
                runtime.transient = None;
                runtime.blocked = None;
                runtime.succeeded_once = true;
                runtime.next_attempt = None;
                runtime.next_attempt_at = None;
                if pending == 0 {
                    if let Err(error) = self.store().record_success(&now_iso()) {
                        tracing::warn!(%error, "date de synchronisation non enregistrée");
                    }
                }
            }
            Some(failure) => self.schedule_retry(&mut runtime, failure, config.provider),
        }
        drop(runtime);
        self.publish_status();
    }

    fn schedule_retry(
        &self,
        runtime: &mut Runtime,
        failure: &SyncFailure,
        provider: SyncProviderKind,
    ) {
        let message = user_message_of(failure, provider);
        let delay = match failure {
            SyncFailure::Remote(RemoteError::Unauthorized(_)) => {
                runtime.blocked = Some(Blocked {
                    phase: SyncPhase::AuthRequired,
                    message,
                });
                return;
            }
            SyncFailure::Incompatible(_) => {
                runtime.blocked = Some(Blocked {
                    phase: SyncPhase::Incompatible,
                    message,
                });
                return;
            }
            SyncFailure::Remote(RemoteError::RateLimited(Some(delay))) => *delay,
            _ => {
                let index = usize::try_from(runtime.failures)
                    .unwrap_or(usize::MAX)
                    .min(BACKOFF.len() - 1);
                Duration::from_secs(BACKOFF[index])
            }
        };
        let phase = match failure {
            SyncFailure::Remote(error) => describe_remote(error, provider).0,
            _ => SyncPhase::TemporaryError,
        };
        runtime.failures = runtime.failures.saturating_add(1);
        runtime.transient = Some((phase, message));
        runtime.next_attempt = Some(Instant::now() + delay);
        runtime.next_attempt_at = Some(
            (chrono::Utc::now() + chrono::TimeDelta::from_std(delay).unwrap_or_default())
                .to_rfc3339(),
        );
        tracing::info!(
            secondes = delay.as_secs(),
            "nouvelle tentative de synchronisation programmée"
        );
    }

    /// Faut-il lancer un cycle maintenant ? Regroupe les modifications et respecte les
    /// délais de reprise ; jamais de boucle serrée.
    fn due(&self, manual: bool) -> bool {
        let Ok(config) = self.store().config() else {
            return false;
        };
        if config.attached().is_none() {
            return false;
        }
        if manual {
            return true;
        }
        if !config.auto_sync {
            return false;
        }
        let watermark = self.store().outbox_watermark().unwrap_or(0);
        let now = Instant::now();
        let mut runtime = self.runtime();
        if runtime.blocked.is_some() {
            return false;
        }
        if runtime.next_attempt.is_some_and(|next| next > now) {
            return false;
        }
        if runtime.next_attempt.is_some() {
            return true;
        }
        let Some(last) = runtime.last_cycle else {
            return true;
        };
        if watermark != runtime.watermark {
            runtime.watermark = watermark;
            runtime.watermark_changed = Some(now);
        }
        let poll = if config.provider == SyncProviderKind::Folder {
            POLL_FOLDER
        } else {
            POLL_CLOUD
        };
        let local_changes = self
            .store()
            .counts()
            .map(|counts| counts.pending > 0)
            .unwrap_or(false);
        if local_changes {
            let calm = runtime
                .watermark_changed
                .is_none_or(|changed| now.duration_since(changed) >= DEBOUNCE);
            let overdue = now.duration_since(last) >= MAX_DELAY;
            if calm || overdue {
                return true;
            }
        }
        now.duration_since(last) >= poll
    }

    /// Tâche de fond : un cycle au démarrage, puis selon les modifications, la relève
    /// périodique et les reprises. S'arrête à la fermeture de l'application.
    pub async fn run(self: Arc<Self>) {
        let mut manual = false;
        loop {
            if self.due(manual) {
                self.run_cycle().await;
            }
            manual = tokio::select! {
                () = self.shutdown.cancelled() => break,
                () = self.notify.notified() => true,
                () = tokio::time::sleep(TICK) => false,
            };
        }
        tracing::info!("tâche de synchronisation arrêtée");
    }
}

fn display_value(value: Option<&serde_json::Value>) -> Option<String> {
    match value? {
        serde_json::Value::Null => None,
        serde_json::Value::String(text) => Some(if text.chars().count() > 280 {
            format!("{}…", text.chars().take(280).collect::<String>())
        } else {
            text.clone()
        }),
        other => Some(other.to_string()),
    }
}

fn record_label(entity: &str, row: Option<&Row>) -> String {
    let field = |name: &str| {
        row.and_then(|row| display_value(row.get(name)))
            .unwrap_or_default()
    };
    let label = match entity {
        "applications" => field("job_title"),
        "companies" | "resume_versions" | "cover_letters" | "saved_views" => field("name"),
        "contacts" => format!("{} {}", field("first_name"), field("name")),
        "status_history" => field("status"),
        "follow_ups" => field("follow_up_date"),
        "interviews" => field("interview_date"),
        "relation_notes" => field("body"),
        "profile" => "Votre profil".to_owned(),
        _ => String::new(),
    };
    let trimmed = label.trim();
    if trimmed.is_empty() {
        "Sans titre".to_owned()
    } else {
        trimmed.chars().take(80).collect()
    }
}

fn present_conflict(conflict: StoredConflict) -> SyncConflict {
    let table = entities::table(&conflict.entity);
    let fields = conflict
        .fields
        .iter()
        .map(|field| ConflictField {
            column: field.column.clone(),
            label: column_label(&field.column).to_owned(),
            local: display_value(
                conflict
                    .local
                    .as_ref()
                    .and_then(|row| row.get(&field.column)),
            ),
            remote: display_value(Some(&field.remote)),
        })
        .collect();
    SyncConflict {
        entity_label: table.map_or("Élément", |table| table.label).to_owned(),
        record_label: record_label(
            &conflict.entity,
            conflict.local.as_ref().or(conflict.remote.data.as_ref()),
        ),
        id: conflict.id,
        entity: conflict.entity,
        kind: conflict.kind,
        detected_at: conflict.detected_at,
        fields,
    }
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
