//! Validation, coffre, sauvegarde et mises à jour.

use crate::core::backup;
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::core::secrets::CoffreSecrets;
use crate::core::updater::{self, UpdateInfo as ReleaseInfo};
use crate::features::ai::domain::{LlmConfig, ProviderKind};
use crate::features::ai::infrastructure::{build_provider, LlmGenerator};
use crate::features::settings::domain::{
    About, AppSettings, LlmForm, Settings, SettingsRepository, UpdateAsset, UpdateInfo,
};
use std::path::{Path, PathBuf};

/// Service des réglages, générique sur le dépôt et le coffre (testable hors trousseau).
pub struct SettingsService<R: SettingsRepository, C: CoffreSecrets> {
    repo: R,
    coffre: C,
    pool: SqlitePool,
    db_path: PathBuf,
}

impl<R: SettingsRepository, C: CoffreSecrets> SettingsService<R, C> {
    #[must_use]
    pub fn new(repo: R, coffre: C, pool: SqlitePool, db_path: PathBuf) -> Self {
        Self {
            repo,
            coffre,
            pool,
            db_path,
        }
    }

    /// Payload les réglages, déplace une clé héritée vers le coffre, puis la réinjecte
    /// pour le formulaire. Ollama ne touche pas au trousseau : les tests CI n'ont pas
    /// de service de secrets.
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
            self.coffre.store_api_key(Some(&heritage))?;
            self.repo.upsert(&settings)?;
            if provider_cloud(&settings.llm.provider) {
                settings.llm.api_key = Some(heritage);
            }
        } else if provider_cloud(&settings.llm.provider) {
            settings.llm.api_key = self.coffre.load_api_key()?;
        }
        Ok(settings.into())
    }

    /// Valide, range la clé dans le coffre, persiste le JSON sans secret.
    ///
    /// # Errors
    /// `Validation` si la configuration est incohérente ; sinon l'erreur du dépôt ou du coffre.
    pub fn save(&self, settings: Settings) -> AppResult<Settings> {
        validate(&settings)?;
        let mut settings = AppSettings::from(settings);
        let cle = settings.llm.api_key.take();
        if provider_cloud(&settings.llm.provider) {
            self.coffre.store_api_key(cle.as_deref())?;
        }
        self.repo.upsert(&settings)?;
        settings.llm.api_key = cle;
        Ok(settings.into())
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
    /// Propage l'erreur SQLite.
    pub fn reset(&self) -> AppResult<()> {
        backup::reset_data(&self.pool)
    }

    #[must_use]
    pub fn about(&self) -> About {
        About {
            version: env!("CARGO_PKG_VERSION").into(),
            name: "Candilog".into(),
        }
    }
}

impl<R: SettingsRepository, C: CoffreSecrets> SettingsService<R, C> {
    /// Teste la connexion au fournisseur décrit par le formulaire, sans le persister.
    ///
    /// # Errors
    /// Retourne l'erreur du fournisseur ou de validation.
    pub async fn test_connection(&self, llm: LlmForm) -> AppResult<()> {
        valider_llm(&llm)?;
        let provider = build_provider(&LlmConfig::from(llm)).await?;
        LlmGenerator::test(provider.as_ref()).await
    }

    /// List les modèles exposés par le fournisseur du formulaire.
    ///
    /// # Errors
    /// Retourne l'erreur du fournisseur ou de validation.
    pub async fn list_models(&self, llm: LlmForm) -> AppResult<Vec<String>> {
        valider_llm(&llm)?;
        let provider = build_provider(&LlmConfig::from(llm)).await?;
        LlmGenerator::list_models(provider.as_ref()).await
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

fn validate(settings: &Settings) -> AppResult<()> {
    valider_llm(&settings.llm)
}

fn valider_llm(llm: &LlmForm) -> AppResult<()> {
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
            if llm.api_key.as_deref().unwrap_or_default().trim().is_empty() {
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
    use crate::core::database::{open_pool, run_local_migrations};
    use crate::core::secrets::CoffreSecrets;
    use crate::features::ai::domain::{AnalysisMode, ProviderKind};
    use crate::features::settings::domain::ThemePref;
    use std::sync::Mutex;

    #[derive(Default)]
    struct CoffreMemoire {
        cle: Mutex<Option<String>>,
    }

    impl CoffreSecrets for CoffreMemoire {
        fn load_api_key(&self) -> AppResult<Option<String>> {
            Ok(self.cle.lock().unwrap().clone())
        }
        fn store_api_key(&self, secret: Option<&str>) -> AppResult<()> {
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
            api_key: None,
            endpoint: Some("http://localhost:11434".into()),
            model: "llama3.2:3b".into(),
            temperature: 0.7,
            mode: AnalysisMode::Auto,
        }
    }

    #[test]
    fn ollama_valide_persiste() {
        let enregistre = service().save(form(ollama())).unwrap();
        assert_eq!(enregistre.language, "fr");
        assert_eq!(enregistre.llm.provider, ProviderKind::Ollama);
    }

    #[test]
    fn cloud_sans_cle_est_refuse() {
        let mut llm = ollama();
        llm.provider = ProviderKind::OpenAI;
        llm.api_key = None;
        llm.model = "gpt-4o".into();
        assert!(matches!(
            service().save(form(llm)),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn custom_sans_endpoint_est_refuse() {
        let mut llm = ollama();
        llm.provider = ProviderKind::Custom("maison".into());
        llm.api_key = Some("k".into());
        llm.endpoint = None;
        llm.model = "x".into();
        assert!(matches!(
            service().save(form(llm)),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn temperature_hors_bornes_est_refusee() {
        let mut llm = ollama();
        llm.temperature = 3.0;
        assert!(matches!(
            service().save(form(llm)),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn la_cle_cloud_quitte_sqlite_vers_le_coffre() {
        let service = service();
        let mut llm = ollama();
        llm.provider = ProviderKind::OpenAI;
        llm.api_key = Some("sk-test".into());
        llm.model = "gpt-4o".into();
        let minutes = service.save(form(llm)).unwrap();
        assert_eq!(minutes.llm.api_key.as_deref(), Some("sk-test"));
        let stored = service.repo.get().unwrap();
        assert!(stored.llm.api_key.is_none());
        assert_eq!(
            service.coffre.load_api_key().unwrap().as_deref(),
            Some("sk-test")
        );
    }

    #[test]
    fn ollama_ne_lit_pas_le_coffre() {
        let service = service();
        service.coffre.store_api_key(Some("sk-cachee")).unwrap();
        let payload = service.load().unwrap();
        assert!(payload.llm.api_key.is_none());
    }
}
