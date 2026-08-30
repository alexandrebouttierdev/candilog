//! Validation, coffre, sauvegarde et mises à jour.

use crate::core::backup;
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::core::secrets::SecretStoreContract;
use crate::core::updater::{self, UpdateInfo as ReleaseInfo};
use crate::features::ai::domain::{LlmConfig, ProviderKind};
use crate::features::ai::infrastructure::{build_provider, LlmGenerator};
use crate::features::settings::domain::{
    About, AppSettings, LlmForm, ResetOutcome, Settings, SettingsRepository, UpdateAsset,
    UpdateInfo,
};
use std::path::{Path, PathBuf};

/// Service des réglages, générique sur le dépôt et le coffre (testable hors trousseau).
pub struct SettingsService<R: SettingsRepository, C: SecretStoreContract> {
    repo: R,
    secret_store: C,
    pool: SqlitePool,
    db_path: PathBuf,
}

impl<R: SettingsRepository, C: SecretStoreContract> SettingsService<R, C> {
    #[must_use]
    pub fn new(repo: R, secret_store: C, pool: SqlitePool, db_path: PathBuf) -> Self {
        Self {
            repo,
            secret_store,
            pool,
            db_path,
        }
    }

    /// Charge les réglages et déplace une éventuelle clé héritée vers le coffre.
    /// La réponse ne contient jamais le secret, seulement son état de configuration.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt ou du coffre.
    pub fn load(&self) -> AppResult<Settings> {
        let mut settings = self.repo.get()?;
        if let Some(heritage) = settings
            .llm
            .api_key
            .take()
            .filter(|cle| !cle.trim().is_empty())
        {
            self.secret_store.store_api_key(Some(&heritage))?;
            self.repo.upsert(&settings)?;
        }
        let api_key_configured = if provider_cloud(&settings.llm.provider) {
            self.secret_store.load_api_key()?.is_some()
        } else {
            false
        };
        Ok(Settings::from_app(settings, api_key_configured))
    }

    /// Valide, range la clé dans le coffre, persiste le JSON sans secret.
    ///
    /// # Errors
    /// `Validation` si la configuration est incohérente ; sinon l'erreur du dépôt ou du coffre.
    pub fn save(&self, settings: Settings, api_key: Option<String>) -> AppResult<Settings> {
        let api_key = non_empty_secret(api_key);
        let stored_api_key = if provider_cloud(&settings.llm.provider) && api_key.is_none() {
            self.secret_store.load_api_key()?
        } else {
            None
        };
        let api_key_configured = api_key.is_some() || stored_api_key.is_some();
        validate(&settings, api_key_configured)?;

        let settings = AppSettings::from(settings);
        if provider_cloud(&settings.llm.provider) {
            if let Some(secret) = api_key.as_deref() {
                self.secret_store.store_api_key(Some(secret))?;
            }
        }
        self.repo.upsert(&settings)?;
        Ok(Settings::from_app(settings, api_key_configured))
    }

    /// Supprime explicitement la clé IA du coffre.
    ///
    /// # Errors
    /// Propage l'erreur du coffre système.
    pub fn clear_api_key(&self) -> AppResult<()> {
        self.secret_store.store_api_key(None)
    }

    /// # Errors
    /// Propage l'erreur SQLite.
    pub fn clear_ai_cache(&self) -> AppResult<()> {
        backup::clear_ai_cache(&self.pool)
    }

    /// # Errors
    /// Propage l'erreur d'export.
    pub fn export(&self, destination: &Path) -> AppResult<()> {
        backup::export(&self.pool, destination)
    }

    /// # Errors
    /// Propage l'erreur de restauration, après retour arrière si besoin.
    pub fn restore(&self, source: &Path) -> AppResult<()> {
        backup::import(&self.pool, &self.db_path, source)
    }

    /// # Errors
    /// Propage l'erreur SQLite. Une indisponibilité du coffre est rapportée dans le résultat,
    /// car les données SQLite ont alors déjà été supprimées de manière irréversible.
    pub fn reset(&self) -> AppResult<ResetOutcome> {
        backup::reset_data(&self.pool)?;
        let secret_cleared = match self.secret_store.store_api_key(None) {
            Ok(()) => true,
            Err(error) => {
                tracing::error!(%error, "données effacées mais secret non supprimé du coffre");
                false
            }
        };
        Ok(ResetOutcome {
            data_cleared: true,
            secret_cleared,
        })
    }

    #[must_use]
    pub fn about(&self) -> About {
        About {
            version: env!("CARGO_PKG_VERSION").into(),
            name: "Candilog".into(),
        }
    }
}

impl<R: SettingsRepository, C: SecretStoreContract> SettingsService<R, C> {
    /// Teste la connexion au fournisseur décrit par le formulaire, sans le persister.
    ///
    /// # Errors
    /// Retourne l'erreur du fournisseur ou de validation.
    pub async fn test_connection(&self, llm: LlmForm, api_key: Option<String>) -> AppResult<()> {
        let config = self.provider_config(llm, api_key)?;
        let provider = build_provider(&config).await?;
        LlmGenerator::test(provider.as_ref()).await
    }

    /// List les modèles exposés par le fournisseur du formulaire.
    ///
    /// # Errors
    /// Retourne l'erreur du fournisseur ou de validation.
    pub async fn list_models(
        &self,
        llm: LlmForm,
        api_key: Option<String>,
    ) -> AppResult<Vec<String>> {
        let config = self.provider_config(llm, api_key)?;
        let provider = build_provider(&config).await?;
        LlmGenerator::list_models(provider.as_ref()).await
    }

    fn provider_config(&self, llm: LlmForm, api_key: Option<String>) -> AppResult<LlmConfig> {
        let mut config = LlmConfig::from(llm);
        if provider_cloud(&config.provider) {
            config.api_key = match non_empty_secret(api_key) {
                Some(secret) => Some(secret),
                None => self.secret_store.load_api_key()?,
            };
        }
        validate_llm(&config, config.api_key.is_some())?;
        Ok(config)
    }

    /// Compare la version installée à la dernière release GitHub.
    ///
    /// # Errors
    /// Retourne une erreur réseau si l'API est inaccessible.
    pub async fn check_update(&self) -> AppResult<Option<UpdateInfo>> {
        let actuelle = updater::version_locale()?;
        let client = updater::client_github()?;
        Ok(updater::check(&client, &actuelle)
            .await?
            .map(UpdateInfo::from))
    }

    /// Télécharge l'installeur puis l'ouvre avec le lanceur système.
    ///
    /// # Errors
    /// Retourne une erreur réseau, d'écriture ou de lancement.
    pub async fn download_update(
        &self,
        url: String,
        name: String,
        notifier: impl FnMut(u8),
    ) -> AppResult<PathBuf> {
        updater::assert_url_installeur_autorisee(&url)?;
        let client = updater::client_github()?;
        let path = updater::download_installeur(&client, &url, &name, notifier).await?;
        updater::ouvrir_file(&path)?;
        Ok(path)
    }
}

impl From<ReleaseInfo> for UpdateInfo {
    fn from(value: ReleaseInfo) -> Self {
        Self {
            version: value.version.to_string(),
            notes: value.notes,
            page_url: value.page_url,
            asset: value.asset.map(|asset| UpdateAsset {
                name: asset.name,
                url: asset.url,
            }),
        }
    }
}

fn provider_cloud(provider: &ProviderKind) -> bool {
    !matches!(provider, ProviderKind::Ollama)
}

fn non_empty_secret(secret: Option<String>) -> Option<String> {
    secret.filter(|value| !value.trim().is_empty())
}

fn validate(settings: &Settings, api_key_configured: bool) -> AppResult<()> {
    let config = LlmConfig::from(settings.llm.clone());
    validate_llm(&config, api_key_configured)
}

fn validate_llm(llm: &LlmConfig, api_key_configured: bool) -> AppResult<()> {
    if !(0.0..=2.0).contains(&llm.temperature) {
        return Err(AppError::Validation(
            "La température doit être comprise entre 0.0 et 2.0".into(),
        ));
    }
    match &llm.provider {
        ProviderKind::Ollama => Ok(()),
        ProviderKind::Custom(_) => {
            if llm
                .endpoint
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                Err(AppError::Validation(
                    "Un endpoint est requis pour un fournisseur personnalisé".into(),
                ))
            } else {
                Ok(())
            }
        }
        ProviderKind::Claude
        | ProviderKind::OpenAI
        | ProviderKind::Gemini
        | ProviderKind::Mistral
        | ProviderKind::Nvidia => {
            if !api_key_configured {
                Err(AppError::Validation(
                    "Une clé API est requise pour ce fournisseur".into(),
                ))
            } else {
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::helpers::connection;
    use crate::core::database::{open_pool, run_local_migrations};
    use crate::core::secrets::SecretStoreContract;
    use crate::features::ai::domain::{AnalysisMode, ProviderKind};
    use crate::features::settings::domain::ThemePref;
    use std::sync::Mutex;

    #[derive(Default)]
    struct CoffreMemoire {
        cle: Mutex<Option<String>>,
        echec_suppression: bool,
    }

    impl SecretStoreContract for CoffreMemoire {
        fn load_api_key(&self) -> AppResult<Option<String>> {
            Ok(self.cle.lock().unwrap().clone())
        }
        fn store_api_key(&self, secret: Option<&str>) -> AppResult<()> {
            if secret.is_none() && self.echec_suppression {
                return Err(AppError::Provider("coffre indisponible".into()));
            }
            *self.cle.lock().unwrap() = secret.filter(|v| !v.trim().is_empty()).map(str::to_owned);
            Ok(())
        }
    }

    struct RepoMemoire {
        store: Mutex<Option<AppSettings>>,
    }

    impl SettingsRepository for RepoMemoire {
        fn get(&self) -> AppResult<AppSettings> {
            Ok(self.store.lock().unwrap().clone().unwrap_or_default())
        }
        fn upsert(&self, settings: &AppSettings) -> AppResult<AppSettings> {
            *self.store.lock().unwrap() = Some(settings.clone());
            Ok(settings.clone())
        }
    }

    fn service() -> SettingsService<RepoMemoire, CoffreMemoire> {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        SettingsService::new(
            RepoMemoire {
                store: Mutex::new(None),
            },
            CoffreMemoire::default(),
            pool,
            PathBuf::new(),
        )
    }

    fn form(llm: LlmForm) -> Settings {
        Settings {
            llm,
            theme: ThemePref::System,
            language: "fr".into(),
        }
    }

    fn ollama() -> LlmForm {
        LlmForm {
            provider: ProviderKind::Ollama,
            api_key_configured: false,
            endpoint: Some("http://localhost:11434".into()),
            model: "llama3.2:3b".into(),
            temperature: 0.7,
            mode: AnalysisMode::Auto,
        }
    }

    #[test]
    fn ollama_valide_persiste() {
        let enregistre = service().save(form(ollama()), None).unwrap();
        assert_eq!(enregistre.language, "fr");
        assert_eq!(enregistre.llm.provider, ProviderKind::Ollama);
    }

    #[test]
    fn cloud_sans_cle_est_refuse() {
        let mut llm = ollama();
        llm.provider = ProviderKind::OpenAI;
        llm.api_key_configured = false;
        llm.model = "gpt-4o".into();
        assert!(matches!(
            service().save(form(llm), None),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn custom_sans_endpoint_est_refuse() {
        let mut llm = ollama();
        llm.provider = ProviderKind::Custom("maison".into());
        llm.endpoint = None;
        llm.model = "x".into();
        assert!(matches!(
            service().save(form(llm), None),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn temperature_hors_bornes_est_refusee() {
        let mut llm = ollama();
        llm.temperature = 3.0;
        assert!(matches!(
            service().save(form(llm), None),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn la_cle_cloud_quitte_sqlite_vers_le_coffre() {
        let service = service();
        let mut llm = ollama();
        llm.provider = ProviderKind::OpenAI;
        llm.api_key_configured = false;
        llm.model = "gpt-4o".into();
        let minutes = service.save(form(llm), Some("sk-test".into())).unwrap();
        assert!(minutes.llm.api_key_configured);
        let stored = service.repo.get().unwrap();
        assert!(stored.llm.api_key.is_none());
        assert_eq!(
            service.secret_store.load_api_key().unwrap().as_deref(),
            Some("sk-test")
        );
    }

    #[test]
    fn ollama_ne_lit_pas_le_coffre() {
        let service = service();
        service
            .secret_store
            .store_api_key(Some("sk-cachee"))
            .unwrap();
        let payload = service.load().unwrap();
        assert!(!payload.llm.api_key_configured);
    }

    #[test]
    fn load_ne_reexpose_jamais_le_secret() {
        let service = service();
        let mut stored = AppSettings::default();
        stored.llm.provider = ProviderKind::OpenAI;
        stored.llm.model = "gpt-4o".into();
        service.repo.upsert(&stored).unwrap();
        service
            .secret_store
            .store_api_key(Some("sk-secret"))
            .unwrap();

        let payload = service.load().unwrap();
        let json = serde_json::to_string(&payload).unwrap();

        assert!(payload.llm.api_key_configured);
        assert!(!json.contains("sk-secret"));
        assert!(!json.contains("api_key\":"));
    }

    #[test]
    fn provider_config_charge_le_secret_du_coffre() {
        let service = service();
        service
            .secret_store
            .store_api_key(Some("sk-stored"))
            .unwrap();
        let mut llm = ollama();
        llm.provider = ProviderKind::OpenAI;
        llm.model = "gpt-4o".into();

        let config = service.provider_config(llm, None).unwrap();

        assert_eq!(config.api_key.as_deref(), Some("sk-stored"));
    }

    #[test]
    fn provider_config_prefere_la_nouvelle_cle() {
        let service = service();
        service
            .secret_store
            .store_api_key(Some("sk-stored"))
            .unwrap();
        let mut llm = ollama();
        llm.provider = ProviderKind::OpenAI;
        llm.model = "gpt-4o".into();

        let config = service
            .provider_config(llm, Some("sk-draft".into()))
            .unwrap();

        assert_eq!(config.api_key.as_deref(), Some("sk-draft"));
    }

    #[test]
    fn clear_api_key_supprime_le_secret() {
        let service = service();
        service
            .secret_store
            .store_api_key(Some("sk-stored"))
            .unwrap();

        service.clear_api_key().unwrap();

        assert_eq!(service.secret_store.load_api_key().unwrap(), None);
    }

    #[test]
    fn reset_supprime_les_donnees_et_le_secret() {
        let service = service();
        connection(&service.pool)
            .unwrap()
            .execute(
                "INSERT INTO app_kv (kv_key, kv_value) VALUES ('test', 'valeur')",
                [],
            )
            .unwrap();
        service
            .secret_store
            .store_api_key(Some("sk-stored"))
            .unwrap();

        let outcome = service.reset().unwrap();

        let remaining: i64 = connection(&service.pool)
            .unwrap()
            .query_row("SELECT COUNT(*) FROM app_kv", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 0);
        assert_eq!(service.secret_store.load_api_key().unwrap(), None);
        assert!(outcome.data_cleared);
        assert!(outcome.secret_cleared);
    }

    #[test]
    fn reset_signale_un_succes_partiel_si_le_coffre_est_indisponible() {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        connection(&pool)
            .unwrap()
            .execute(
                "INSERT INTO app_kv (kv_key, kv_value) VALUES ('test', 'valeur')",
                [],
            )
            .unwrap();
        let service = SettingsService::new(
            RepoMemoire {
                store: Mutex::new(None),
            },
            CoffreMemoire {
                cle: Mutex::new(Some("sk-stored".into())),
                echec_suppression: true,
            },
            pool,
            PathBuf::new(),
        );

        let outcome = service.reset().unwrap();

        let remaining: i64 = connection(&service.pool)
            .unwrap()
            .query_row("SELECT COUNT(*) FROM app_kv", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 0);
        assert!(outcome.data_cleared);
        assert!(!outcome.secret_cleared);
    }
}
