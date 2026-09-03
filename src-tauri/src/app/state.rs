//! État applicatif partagé, construit une fois au démarrage et injecté par Tauri.

use crate::core::config::AppPaths;
use crate::core::database::{open_pool, run_local_migrations, validate_database_file, SqlitePool};
use crate::core::errors::AppResult;
use crate::core::secrets::SecretStore;
use crate::features::ai::application::AiService;
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
pub type Documents = Arc<DocumentsService<SqliteResumeRepository, SqliteCoverLetterRepository>>;
/// Service des entretiens tel que partagé par les commandes.
pub type Interviews = Arc<InterviewService<SqliteInterviewRepository>>;
/// Orchestrateur des traitements IA et de leur annulation.
pub type Ai = Arc<AiService>;
/// Réglages, coffre, sauvegardes et mises à jour.
pub type SettingsHandle = Arc<SettingsService<SqliteSettingsRepository, SecretStore>>;
/// Service du profil professionnel.
pub type Profile = Arc<ProfileService<SqliteProfileRepository>>;
/// Service des relances tel que partagé par les commandes.
pub type FollowUps = Arc<FollowUpService<SqliteFollowUpRepository>>;
/// Service des référentiels métier tel que partagé par les commandes.
pub type Referentials = Arc<ReferentialService<SqliteReferentialRepository>>;

/// Dépendances partagées par toutes les commandes.
///
/// Un unique exemplaire est construit au démarrage puis confié à Tauri via `manage` : les
/// commandes le reçoivent en `State<'_, AppState>` et ne recréent jamais ni connexion, ni
/// dépôt, ni client HTTP (docs/ARCHITECTURE.md).
///
/// Les services sont derrière `Arc` parce qu'une commande `async` doit s'approprier ce
/// qu'elle déplace vers `spawn_blocking` : elle ne peut pas y emprunter l'état, dont la
/// durée de vie est liée à l'appel.
///
/// D'autres services s'ajoutent au fil des tranches de migration.
pub struct AppState {
    /// Service du tableau de bord et des analyses.
    pub analytics: Analytics,
    /// Service des candidatures.
    pub applications: Applications,
    /// Service des entreprises.
    pub companies: Companies,
    /// Service des contacts du réseau.
    pub contacts: Contacts,
    /// Bibliothèques locales de CV et lettres.
    pub documents: Documents,
    /// Service des entretiens.
    pub interviews: Interviews,
    /// Analysis et génération de documents.
    pub ai: Ai,
    /// Réglages applicatifs et maintenance.
    pub settings: SettingsHandle,
    /// Service du profil professionnel.
    pub profile: Profile,
    /// Service des relances.
    pub followups: FollowUps,
    /// Service des quatre référentiels métier.
    pub referentials: Referentials,
    /// Pool `SQLite` local.
    pub sqlite: SqlitePool,
    /// Path du fichier de base, nécessaire à l'export et à la restauration de sauvegarde.
    pub db_path: PathBuf,
}

impl AppState {
    /// Construit l'état sur le fichier de données de l'utilisateur et applique les migrations.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si le pool ne peut pas être ouvert ou si une migration
    /// échoue, et `AppError::Validation` si le dossier de données est introuvable.
    pub fn persistent() -> AppResult<Self> {
        let paths = AppPaths::discover()?;
        validate_database_file(&paths.database)?;
        let pool = open_pool(Some(&paths.database))?;
        run_local_migrations(&pool)?;
        // Le fichier de base n'existe pas encore au moment où les chemins sont résolus :
        // ses permissions ne peuvent être restreintes qu'une fois la base ouverte.
        paths.securiser();
        Self::sur_pool(pool, paths.database, paths.photos_dir)
    }

    /// Construit l'état sur une base **en mémoire**, réservé aux tests.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si le pool `SQLite` ne peut pas être initialisé.
    pub fn in_memory() -> AppResult<Self> {
        let pool = open_pool(None)?;
        run_local_migrations(&pool)?;
        // Dossier de photos propre à l'instance : deux états en mémoire ne doivent pas se
        // marcher dessus, et rien ne subsiste entre deux exécutions de la suite.
        let photos_dir =
            std::env::temp_dir().join(format!("candilog-photos-{}", uuid::Uuid::new_v4()));
        Self::sur_pool(pool, PathBuf::new(), photos_dir)
    }

    /// Assemble dépôts et services autour d'un pool déjà migré.
    fn sur_pool(pool: SqlitePool, db_path: PathBuf, photos_dir: PathBuf) -> AppResult<Self> {
        // Les référentiels sont semés par `init_schema.sql` : aucune étape d'amorçage n'est
        // nécessaire ici, et les listes sont donc identiques d'une installation à l'autre.
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
            documents: Arc::new(DocumentsService::new(
                SqliteResumeRepository::new(pool.clone()),
                SqliteCoverLetterRepository::new(pool.clone()),
            )),
            interviews: Arc::new(InterviewService::new(SqliteInterviewRepository::new(
                pool.clone(),
            ))),
            ai: Arc::new(AiService::new(pool.clone())),
            settings: Arc::new(SettingsService::new(
                SqliteSettingsRepository::new(pool.clone()),
                SecretStore,
                pool.clone(),
                db_path.clone(),
            )),
            profile: Arc::new(ProfileService::new(
                SqliteProfileRepository::new(pool.clone()),
                photos_dir,
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
