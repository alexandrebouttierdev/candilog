//! État applicatif partagé, construit une fois au démarrage et injecté par Tauri.

use crate::core::config::AppPaths;
use crate::core::database::{open_pool, run_local_migrations, validate_database_file, SqlitePool};
use crate::core::errors::AppResult;
use crate::core::secrets::SecretStore;
use crate::features::ai::application::{AiService, ManagedOllamaPaths, ManagedOllamaService};
use crate::features::analytics::application::AnalyticsService;
use crate::features::analytics::infrastructure::SqliteAnalyticsRepository;
use crate::features::applications::application::ApplicationService;
use crate::features::applications::infrastructure::SqliteApplicationRepository;
use crate::features::companies::application::CompanyService;
use crate::features::companies::infrastructure::SqliteCompanyRepository;
use crate::features::contacts::application::ContactService;
use crate::features::contacts::infrastructure::SqliteContactRepository;
use crate::features::documents::application::DocumentsService;
use crate::features::documents::infrastructure::{
    SqliteCoverLetterRepository, SqliteResumeRepository,
};
use crate::features::followups::application::FollowUpService;
use crate::features::followups::infrastructure::SqliteFollowUpRepository;
use crate::features::interviews::application::InterviewService;
use crate::features::interviews::infrastructure::SqliteInterviewRepository;
use crate::features::profile::application::ProfileService;
use crate::features::profile::infrastructure::SqliteProfileRepository;
use crate::features::referentials::application::ReferentialService;
use crate::features::referentials::infrastructure::SqliteReferentialRepository;
use crate::features::settings::application::SettingsService;
use crate::features::settings::infrastructure::SqliteSettingsRepository;
use std::path::PathBuf;
use std::sync::Arc;

/// Service des candidatures tel que partagé par les commandes.
pub type Applications = Arc<ApplicationService<SqliteApplicationRepository>>;
/// Service du tableau de bord et des analyses.
pub type Analytics = Arc<AnalyticsService<SqliteAnalyticsRepository>>;
/// Service des entreprises tel que partagé par les commandes.
pub type Companies = Arc<CompanyService<SqliteCompanyRepository>>;
/// Service des contacts tel que partagé par les commandes.
pub type Contacts = Arc<ContactService<SqliteContactRepository>>;
/// Service des bibliothèques de CV et lettres.
pub type Documents = Arc<
    DocumentsService<SqliteResumeRepository, SqliteCoverLetterRepository, SqliteProfileRepository>,
>;
/// Service des entretiens tel que partagé par les commandes.
pub type Interviews = Arc<InterviewService<SqliteInterviewRepository>>;
/// Orchestrateur des traitements IA et de leur annulation.
pub type Ai = Arc<AiService>;
/// Runtime Ollama privé géré par Candilog.
pub type ManagedOllama = Arc<ManagedOllamaService>;
/// Réglages, coffre, sauvegardes et mises à jour.
pub type SettingsHandle = Arc<SettingsService<SqliteSettingsRepository, SecretStore>>;
/// Service du profil professionnel.
pub type Profile = Arc<ProfileService<SqliteProfileRepository>>;
/// Service des relances tel que partagé par les commandes.
pub type FollowUps = Arc<FollowUpService<SqliteFollowUpRepository>>;
/// Service des référentiels métier tel que partagé par les commandes.
pub type Referentials = Arc<ReferentialService<SqliteReferentialRepository>>;

/// Dépendances partagées par toutes les commandes.
pub struct AppState {
    pub analytics: Analytics,
    pub applications: Applications,
    pub companies: Companies,
    pub contacts: Contacts,
    pub documents: Documents,
    pub interviews: Interviews,
    pub ai: Ai,
    pub managed_ollama: ManagedOllama,
    pub settings: SettingsHandle,
    pub profile: Profile,
    pub followups: FollowUps,
    pub referentials: Referentials,
    pub sqlite: SqlitePool,
    pub db_path: PathBuf,
}

impl AppState {
    pub fn persistent() -> AppResult<Self> {
        let paths = AppPaths::discover()?;
        validate_database_file(&paths.database)?;
        let pool = open_pool(Some(&paths.database))?;
        run_local_migrations(&pool)?;
        paths.securiser();
        Self::sur_pool(
            pool,
            paths.database,
            paths.photos_dir,
            ManagedOllamaPaths {
                runtime_root: paths.managed_ollama_runtime_dir,
                models_dir: paths.managed_ollama_models_dir,
                downloads_dir: paths.managed_ollama_downloads_dir,
            },
        )
    }

    pub fn in_memory() -> AppResult<Self> {
        let pool = open_pool(None)?;
        run_local_migrations(&pool)?;
        let photos_dir =
            std::env::temp_dir().join(format!("candilog-photos-{}", uuid::Uuid::new_v4()));
        let managed_root =
            std::env::temp_dir().join(format!("candilog-managed-ollama-{}", uuid::Uuid::new_v4()));
        Self::sur_pool(
            pool,
            PathBuf::new(),
            photos_dir,
            ManagedOllamaPaths {
                runtime_root: managed_root.join("runtime"),
                models_dir: managed_root.join("models"),
                downloads_dir: managed_root.join("downloads"),
            },
        )
    }

    fn sur_pool(
        pool: SqlitePool,
        db_path: PathBuf,
        photos_dir: PathBuf,
        managed_paths: ManagedOllamaPaths,
    ) -> AppResult<Self> {
        let managed_ollama = Arc::new(ManagedOllamaService::new(pool.clone(), managed_paths));
        let profile = Arc::new(ProfileService::new(
            SqliteProfileRepository::new(pool.clone()),
            photos_dir,
        ));
        Ok(Self {
            analytics: Arc::new(AnalyticsService::new(SqliteAnalyticsRepository::new(
                pool.clone(),
            ))),
            applications: Arc::new(ApplicationService::new(SqliteApplicationRepository::new(
                pool.clone(),
            ))),
            companies: Arc::new(CompanyService::new(SqliteCompanyRepository::new(
                pool.clone(),
            ))),
            contacts: Arc::new(ContactService::new(SqliteContactRepository::new(
                pool.clone(),
            ))),
            profile: Arc::clone(&profile),
            documents: Arc::new(DocumentsService::new(
                SqliteResumeRepository::new(pool.clone()),
                SqliteCoverLetterRepository::new(pool.clone()),
                profile,
            )),
            interviews: Arc::new(InterviewService::new(SqliteInterviewRepository::new(
                pool.clone(),
            ))),
            ai: Arc::new(AiService::new(pool.clone(), Arc::clone(&managed_ollama))),
            managed_ollama,
            settings: Arc::new(SettingsService::new(
                SqliteSettingsRepository::new(pool.clone()),
                SecretStore,
                pool.clone(),
                db_path.clone(),
            )),
            followups: Arc::new(FollowUpService::new(SqliteFollowUpRepository::new(
                pool.clone(),
            ))),
            referentials: Arc::new(ReferentialService::new(SqliteReferentialRepository::new(
                pool.clone(),
            ))),
            sqlite: pool,
            db_path,
        })
    }
}
