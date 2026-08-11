//! État applicatif global : services métier, pool SQLite et coffre à secrets.

use crate::modules::candidatures::repository::SqliteCandidatureRepository;
use crate::modules::candidatures::service::CandidatureService;
use crate::modules::contacts::repository::SqliteContactRepository;
use crate::modules::contacts::service::ContactService;
use crate::modules::cv::repository::SqliteCvVersionRepository;
use crate::modules::cv::service::CvVersionService;
use crate::modules::entreprises::repository::SqliteEntrepriseRepository;
use crate::modules::entreprises::service::EntrepriseService;
use crate::modules::entretiens::repository::SqliteEntretienRepository;
use crate::modules::entretiens::service::EntretienService;
use crate::modules::ia::cache::SqliteCacheIaRepository;
pub use crate::modules::metriques::model::{AppelLlm, OperationLlm, OrigineScore, ScoreAts};
pub use crate::modules::metriques::repository::{MetriquesRepository, SqliteMetriquesRepository};
use crate::modules::profil::repository::SqliteProfilRepository;
use crate::modules::profil::service::ProfilService;
use crate::modules::relances::repository::SqliteRelanceRepository;
use crate::modules::relances::service::RelanceService;
pub use crate::modules::settings::model::AppSettings;
use crate::modules::settings::repository::SqliteSettingsRepository;
use crate::modules::settings::service::SettingsService;
use crate::shared::db::{open_pool, run_local_migrations, SqlitePool};
use crate::shared::error::AppResult;
use crate::shared::secrets::SecretStore;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

/// État partagé de l'application (construit une fois au démarrage).
pub struct AppState {
    /// Service des paramètres.
    pub settings: SettingsService<SqliteSettingsRepository>,
    /// Service du profil.
    pub profil: ProfilService<SqliteProfilRepository>,
    /// Service des versions de CV.
    pub cv: CvVersionService<SqliteCvVersionRepository>,
    /// Service des candidatures.
    pub candidatures: CandidatureService<SqliteCandidatureRepository>,
    /// Service des entreprises.
    pub entreprises: EntrepriseService<SqliteEntrepriseRepository>,
    /// Service des contacts (réseau).
    pub contacts: ContactService<SqliteContactRepository>,
    /// Service des relances.
    pub relances: RelanceService<SqliteRelanceRepository>,
    /// Service des entretiens.
    pub entretiens: EntretienService<SqliteEntretienRepository>,
    /// Dépôt des métriques locales (télémétrie `LLM` + historique `ATS`, `SQLite`).
    pub metriques: SqliteMetriquesRepository,
    /// Cache local des résultats d'analyse `IA` (`SQLite`).
    pub cache_ia: SqliteCacheIaRepository,
    /// Pool `SQLite` local.
    pub sqlite: SqlitePool,
    /// Chemin du fichier de base de données `SQLite` (pour l'export/import de backup).
    pub db_path: PathBuf,
    /// Coffre natif contenant la clé API IA.
    pub secrets: SecretStore,
    /// Générations `IA` en cours, indexées par identifiant client, pour l'annulation réelle.
    /// Chaque jeton, une fois annulé, interrompt le `select!` de la commande (abandon du futur
    /// reqwest = coupure de la connexion HTTP au fournisseur).
    pub generations: Mutex<HashMap<String, CancellationToken>>,
}

impl AppState {
    /// Construit l'état applicatif sur une base `SQLite` **en mémoire** : réservé aux tests.
    ///
    /// Le binaire passe par [`AppState::persistent`], seul chemin qui ouvre le fichier de
    /// l'utilisateur.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si le pool `SQLite` ne peut pas être initialisé.
    pub fn new() -> AppResult<Self> {
        Self::with_database(None, PathBuf::new())
    }

    /// Construit l'état avec une base `SQLite` persistante.
    ///
    /// # Errors
    /// Retourne une erreur si le fichier ou ses migrations ne peuvent pas être ouverts.
    pub fn persistent(database_path: &std::path::Path) -> AppResult<Self> {
        Self::with_database(Some(database_path), database_path.to_path_buf())
    }

    fn with_database(database_path: Option<&std::path::Path>, db_path: PathBuf) -> AppResult<Self> {
        let sqlite = open_pool(database_path)?;
        run_local_migrations(&sqlite)?;
        let settings = SettingsService::new(SqliteSettingsRepository::new(sqlite.clone()));
        let profil = ProfilService::new(SqliteProfilRepository::new(sqlite.clone()));
        let cv = CvVersionService::new(SqliteCvVersionRepository::new(sqlite.clone()));
        let candidatures =
            CandidatureService::new(SqliteCandidatureRepository::new(sqlite.clone()));
        let entreprises = EntrepriseService::new(SqliteEntrepriseRepository::new(sqlite.clone()));
        let contacts = ContactService::new(SqliteContactRepository::new(sqlite.clone()));
        let relances = RelanceService::new(SqliteRelanceRepository::new(sqlite.clone()));
        let entretiens = EntretienService::new(SqliteEntretienRepository::new(sqlite.clone()));
        let metriques = SqliteMetriquesRepository::new(sqlite.clone());
        let cache_ia = SqliteCacheIaRepository::new(sqlite.clone());
        Ok(Self {
            settings,
            profil,
            cv,
            candidatures,
            entreprises,
            contacts,
            relances,
            entretiens,
            metriques,
            cache_ia,
            sqlite,
            db_path,
            secrets: SecretStore,
            generations: Mutex::new(HashMap::new()),
        })
    }

    /// Enregistre une génération `IA` en cours et renvoie son jeton d'annulation.
    ///
    /// Un `generation_id` déjà présent voit son jeton remplacé (nouvelle génération pour le même
    /// identifiant) ; l'ancienne, si elle tourne encore, se terminera d'elle-même.
    pub fn register_generation(&self, generation_id: &str) -> CancellationToken {
        let token = CancellationToken::new();
        let mut guard = self
            .generations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.insert(generation_id.to_string(), token.clone());
        token
    }

    /// Annule la génération identifiée (sans effet si inconnue ou déjà terminée).
    pub fn cancel_generation(&self, generation_id: &str) {
        let guard = self
            .generations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(token) = guard.get(generation_id) {
            token.cancel();
        }
    }

    /// Retire une génération terminée du registre. À appeler dans **toutes** les branches de sortie
    /// de la commande de génération (guard) pour éviter toute fuite du registre.
    pub fn finish_generation(&self, generation_id: &str) {
        let mut guard = self
            .generations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.remove(generation_id);
    }

    /// Charge les préférences (base locale) et réinjecte la clé IA depuis le coffre natif.
    /// Migre automatiquement une ancienne clé encore présente dans les préférences elles-mêmes
    /// (résidu d'avant l'introduction du coffre natif) vers le coffre.
    ///
    /// # Errors
    /// Propage les erreurs de persistance ou du coffre système.
    pub fn secure_settings(&self) -> AppResult<crate::modules::settings::model::AppSettings> {
        let mut settings = self.settings.get()?;
        if let Some(legacy_key) = settings.llm.api_key.take() {
            self.secrets.store_api_key(Some(&legacy_key))?;
            self.settings.persist(&settings)?;
            settings.llm.api_key = Some(legacy_key);
        } else {
            settings.llm.api_key = self.secrets.load_api_key()?;
        }
        Ok(settings)
    }

    /// Valide puis sauvegarde les préférences (base locale) sans jamais persister la clé IA en
    /// clair : elle est déplacée vers le coffre natif du système.
    ///
    /// # Errors
    /// Propage les erreurs de validation, du coffre ou de persistance.
    pub async fn update_secure_settings(
        &self,
        mut settings: crate::modules::settings::model::AppSettings,
    ) -> AppResult<crate::modules::settings::model::AppSettings> {
        let submitted_key = settings
            .llm
            .api_key
            .take()
            .filter(|key| !key.trim().is_empty());
        let effective_key = match submitted_key {
            Some(key) => Some(key),
            None => self.secrets.load_api_key()?,
        };
        settings.llm.api_key = effective_key.clone();
        SettingsService::<SqliteSettingsRepository>::validate(&settings)?;
        crate::shared::llm::validate_llm_endpoint(&settings.llm).await?;
        self.secrets.store_api_key(effective_key.as_deref())?;
        settings.llm.api_key = None;
        let mut saved = self.settings.persist(&settings)?;
        saved.llm.api_key = None;
        Ok(saved)
    }
}

#[cfg(test)]
#[path = "tests/state/mod.rs"]
mod tests;
