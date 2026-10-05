//! Validation, coffre, sauvegarde et mises à jour.

use crate::core::backup;
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::core::secrets::SecretStoreContract;
use crate::core::updater::{self, UpdateInfo as ReleaseInfo};
use crate::core::utils::validation::validate_optional_http_url;
use crate::features::ai::domain::{LlmConfig, ProviderKind};
use crate::features::ai::infrastructure::{build_provider, LlmGenerator};
use crate::features::settings::domain::{
    About, AppSettings, LlmForm, ResetOutcome, Settings, SettingsRepository, UpdateAsset,
    UpdateInfo,
};
use std::path::{Path, PathBuf};

/// Service des réglages, générique sur le dépôt et le coffre (testable hors trousseau).
/// Faut-il réinterroger GitHub ?
///
/// Oui quand l'application ne l'a jamais fait, quand `DELAI_VERIFICATION_MAJ` est écoulé,
/// quand l'horodatage retenu est illisible — rester bloqué pour toujours sur une valeur
/// qu'aucun code ne sait plus relire serait pire que de vérifier une fois de trop — et
/// quand il est dans le futur, ce qui ne peut venir que d'une horloge déréglée depuis.
fn verification_maj_due(derniere: Option<&str>, maintenant: chrono::DateTime<chrono::Utc>) -> bool {
    let Some(texte) = derniere else {
        return true;
    };
    let Ok(instant) = chrono::DateTime::parse_from_rfc3339(texte) else {
        tracing::info!(horodatage = texte, "horodatage de vérification illisible");
        return true;
    };
    let ecoule = maintenant - instant.with_timezone(&chrono::Utc);
    ecoule.num_seconds() < 0 || ecoule >= DELAI_VERIFICATION_MAJ
}

/// Délai minimal entre deux interrogations de l'API GitHub au démarrage.
///
/// Un jour : l'application est ouverte au moins une fois par jour par qui cherche un emploi,
/// ce qui suffit à voir une publication sans marteler l'API — GitHub plafonne les clients
/// anonymes à soixante requêtes par heure et par adresse.
const DELAI_VERIFICATION_MAJ: chrono::TimeDelta = chrono::TimeDelta::hours(24);

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
        let provider_id = settings.llm.provider.storage_id().to_string();
        if let Some(heritage) = settings
            .llm
            .api_key
            .take()
            .filter(|cle| !cle.trim().is_empty())
        {
            self.secret_store
                .store_api_key(&provider_id, Some(&heritage))?;
            self.repo.upsert(&settings)?;
        }
        settings.capture_llm_preset();
        let keys_configured = self.keys_configured(&settings)?;
        let api_key_configured = if provider_cloud(&settings.llm.provider) {
            keys_configured.get(&provider_id).copied().unwrap_or(false)
        } else {
            false
        };
        Ok(Settings::from_app(
            settings,
            api_key_configured,
            &keys_configured,
        ))
    }

    /// Valide, range la clé dans le coffre du fournisseur actif, persiste le JSON sans secret.
    ///
    /// Les presets des autres fournisseurs déjà connus (envoyés par le frontend ou déjà
    /// stockés) sont conservés : enregistrer Mistral n'efface pas la config OpenAI.
    ///
    /// # Errors
    /// `Validation` si la configuration est incohérente ; sinon l'erreur du dépôt ou du coffre.
    pub fn save(&self, settings: Settings, api_key: Option<String>) -> AppResult<Settings> {
        let provider_id = settings.llm.provider.storage_id().to_string();
        let api_key = non_empty_secret(api_key);
        let stored_api_key = if provider_cloud(&settings.llm.provider) && api_key.is_none() {
            self.secret_store.load_api_key(&provider_id)?
        } else {
            None
        };
        let api_key_configured = api_key.is_some() || stored_api_key.is_some();
        validate(&settings, api_key_configured)?;

        let existing = self.repo.get()?;
        let mut app = AppSettings::from(settings);
        // Fusionne : presets déjà persistés ← presets du formulaire ← fournisseur actif.
        let mut presets = existing.llm_presets;
        for (id, preset) in std::mem::take(&mut app.llm_presets) {
            presets.insert(id, preset);
        }
        app.llm_presets = presets;
        app.capture_llm_preset();
        // Les métadonnées du modèle sont gérées par le service IA local : sauvegarder le
        // formulaire général ne doit ni les exposer au frontend ni les réinitialiser.
        app.managed_ollama = existing.managed_ollama;
        if provider_cloud(&app.llm.provider) {
            if let Some(secret) = api_key.as_deref() {
                self.secret_store
                    .store_api_key(&provider_id, Some(secret))?;
            }
        }
        self.repo.upsert(&app)?;
        let keys_configured = self.keys_configured(&app)?;
        Ok(Settings::from_app(
            app,
            api_key_configured,
            &keys_configured,
        ))
    }

    /// Supprime explicitement la clé du fournisseur actuellement actif.
    ///
    /// # Errors
    /// Propage l'erreur du coffre système.
    pub fn clear_api_key(&self) -> AppResult<()> {
        let provider_id = self.repo.get()?.llm.provider.storage_id().to_string();
        self.secret_store.store_api_key(&provider_id, None)
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
        let secret_cleared = match self.secret_store.clear_all_api_keys() {
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

    /// Indique, pour chaque preset connu, si une clé est présente dans le coffre.
    fn keys_configured(
        &self,
        settings: &AppSettings,
    ) -> AppResult<std::collections::BTreeMap<String, bool>> {
        let mut map = std::collections::BTreeMap::new();
        for id in settings.llm_presets.keys() {
            let configured = self.secret_store.load_api_key(id)?.is_some();
            map.insert(id.clone(), configured);
        }
        let active = settings.llm.provider.storage_id().to_string();
        if !map.contains_key(&active) && provider_cloud(&settings.llm.provider) {
            map.insert(
                active,
                self.secret_store
                    .load_api_key(settings.llm.provider.storage_id())?
                    .is_some(),
            );
        }
        Ok(map)
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

    /// Liste les modèles exposés par le fournisseur du formulaire.
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

    /// Construit la configuration d'un essai de connexion à partir du formulaire.
    ///
    /// La clé du coffre n'est reprise que si le formulaire décrit **un couple
    /// (fournisseur, endpoint) déjà mémorisé** — soit le fournisseur actif, soit un preset
    /// enregistré. Sans ce contrôle, un appel IPC forgé pouvait déclarer un fournisseur
    /// personnalisé pointant vers une adresse quelconque et se faire présenter le secret.
    ///
    /// # Errors
    /// `Validation` si le formulaire vise un autre fournisseur ou un autre endpoint sans
    /// fournir explicitement la clé à utiliser.
    fn provider_config(&self, llm: LlmForm, api_key: Option<String>) -> AppResult<LlmConfig> {
        let mut config = LlmConfig::from(llm);
        let provider_id = config.provider.storage_id();
        if provider_cloud(&config.provider) {
            config.api_key = match non_empty_secret(api_key) {
                Some(secret) => Some(secret),
                None if self.peut_reutiliser_cle(&config)? => {
                    self.secret_store.load_api_key(provider_id)?
                }
                None => {
                    tracing::warn!(
                        endpoint = config.endpoint_effectif(),
                        "essai de connexion vers un fournisseur non enregistré, clé du coffre non transmise"
                    );
                    return Err(AppError::Validation(
                        "Saisissez la clé API à utiliser pour ce fournisseur avant de tester la connexion".into(),
                    ));
                }
            };
        }
        validate_llm(&config, config.api_key.is_some())?;
        Ok(config)
    }

    /// Le formulaire vise-t-il un fournisseur/endpoint pour lequel une clé a déjà été rangée ?
    fn peut_reutiliser_cle(&self, config: &LlmConfig) -> AppResult<bool> {
        let enregistres = self.repo.get()?;
        if enregistres.llm.provider == config.provider
            && enregistres.llm.endpoint_effectif() == config.endpoint_effectif()
        {
            return Ok(true);
        }
        let id = config.provider.storage_id();
        Ok(enregistres.llm_presets.get(id).is_some_and(|preset| {
            preset.endpoint_effectif(&config.provider) == config.endpoint_effectif()
        }))
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

    /// Vérification au démarrage : interroge GitHub **une fois par jour au plus**.
    ///
    /// Retourne `None` sans aucun appel réseau tant que `DELAI_VERIFICATION_MAJ` n'est pas
    /// écoulé depuis la dernière tentative. L'instant est retenu **avant** l'appel et quelle
    /// qu'en soit l'issue : sinon une panne réseau ou un quota GitHub ferait réessayer à
    /// chaque lancement, exactement quand il ne faut pas insister.
    ///
    /// Une vérification qui échoue ne remonte pas : hors ligne, API indisponible ou quota
    /// dépassé ne sont pas des problèmes que l'utilisateur doive traiter au lancement, et
    /// l'écran Mises à jour reste là pour une vérification explicite — qui, elle, rapporte
    /// son erreur.
    ///
    /// # Errors
    /// Retourne une erreur si l'horodatage ne peut être ni lu ni écrit.
    pub async fn check_update_if_due(&self) -> AppResult<Option<UpdateInfo>> {
        let maintenant = chrono::Utc::now();
        if !verification_maj_due(self.repo.last_update_check()?.as_deref(), maintenant) {
            return Ok(None);
        }
        self.repo.mark_update_check(&maintenant.to_rfc3339())?;
        match self.check_update().await {
            Ok(disponible) => Ok(disponible),
            Err(erreur) => {
                tracing::info!(%erreur, "vérification de mise à jour au démarrage sans réponse");
                Ok(None)
            }
        }
    }

    /// Télécharge l'installeur de la dernière release, vérifie son empreinte, puis l'ouvre.
    ///
    /// L'asset est **re-résolu ici** à partir de l'API GitHub : le frontend ne transmet plus
    /// ni URL ni nom de fichier. Une commande IPC forgée ne peut donc plus désigner ce qui
    /// sera téléchargé puis confié au lanceur système.
    ///
    /// # Errors
    /// Retourne une erreur réseau, d'empreinte, d'écriture ou de lancement, et
    /// `Validation` si aucune mise à jour n'est disponible pour cette plateforme.
    pub async fn download_update(&self, notifier: impl FnMut(u8)) -> AppResult<PathBuf> {
        let actuelle = updater::version_locale()?;
        let client = updater::client_github()?;
        let release = updater::check(&client, &actuelle)
            .await?
            .ok_or_else(|| AppError::Validation("Aucune mise à jour n'est disponible.".into()))?;
        let asset = release.asset.ok_or_else(|| {
            AppError::Validation(
                "Aucun installateur n'est publié pour ce système dans cette release.".into(),
            )
        })?;
        let path = updater::download_installeur(
            &client,
            &release.assets,
            &asset.url,
            &asset.name,
            notifier,
        )
        .await?;
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
    !matches!(provider, ProviderKind::Ollama | ProviderKind::CandilogLocal)
}

fn non_empty_secret(secret: Option<String>) -> Option<String> {
    secret.filter(|value| !value.trim().is_empty())
}

/// Fournisseurs qui peuvent recevoir un consentement d'envoi : tous sauf l'IA locale.
/// Ollama et « personnalisé » y figurent — ils sont distants dès que leur adresse l'est.
const REMOTE_PROVIDER_IDS: [&str; 7] = [
    "ollama", "claude", "openai", "gemini", "mistral", "deepseek", "custom",
];

fn validate(settings: &Settings, api_key_configured: bool) -> AppResult<()> {
    let config = LlmConfig::from(settings.llm.clone());
    validate_llm(&config, api_key_configured)?;
    // Une route nomme toujours un modèle : une route vide n'est ni « suivre le principal »
    // (route absente) ni « désactivée » (`null`), et la tâche échouerait sans explication.
    // Un consentement ne vaut que pour un fournisseur distant connu : une liste forgée ne
    // doit ni grossir les réglages, ni nommer l'IA locale, qui n'envoie rien.
    if settings.remote_send_consents.len() > REMOTE_PROVIDER_IDS.len()
        || settings
            .remote_send_consents
            .iter()
            .any(|id| !REMOTE_PROVIDER_IDS.contains(&id.as_str()))
    {
        return Err(AppError::Validation(
            "Les confirmations d'envoi distant enregistrées sont invalides.".into(),
        ));
    }
    for (task, route) in &settings.ai_routes {
        if let Some(route) = route {
            let model = route.model.trim();
            if model.is_empty() || model.len() > 200 {
                return Err(AppError::Validation(format!(
                    "Choisissez un modèle pour « {} ».",
                    task.label()
                )));
            }
        }
    }
    Ok(())
}

fn validate_llm(llm: &LlmConfig, api_key_configured: bool) -> AppResult<()> {
    if !(0.0..=2.0).contains(&llm.temperature) {
        return Err(AppError::Validation(
            "La température doit être comprise entre 0.0 et 2.0".into(),
        ));
    }
    match &llm.provider {
        ProviderKind::Ollama | ProviderKind::CandilogLocal => Ok(()),
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
                validate_optional_http_url(llm.endpoint.as_deref(), "L'endpoint")
            }
        }
        ProviderKind::Claude
        | ProviderKind::OpenAI
        | ProviderKind::Gemini
        | ProviderKind::Mistral
        | ProviderKind::DeepSeek => {
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
    use crate::features::ai::domain::{AiTask, AnalysisMode, ProviderKind, TaskRoute};
    use crate::features::settings::domain::ThemePref;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct CoffreMemoire {
        cles: Mutex<HashMap<String, String>>,
        echec_suppression: bool,
    }

    impl SecretStoreContract for CoffreMemoire {
        fn load_api_key(&self, provider_id: &str) -> AppResult<Option<String>> {
            Ok(self.cles.lock().unwrap().get(provider_id).cloned())
        }
        fn store_api_key(&self, provider_id: &str, secret: Option<&str>) -> AppResult<()> {
            if secret.is_none() && self.echec_suppression {
                return Err(AppError::Provider("coffre indisponible".into()));
            }
            let mut cles = self.cles.lock().unwrap();
            match secret.filter(|v| !v.trim().is_empty()) {
                Some(secret) => {
                    cles.insert(provider_id.to_string(), secret.to_owned());
                }
                None => {
                    cles.remove(provider_id);
                }
            }
            Ok(())
        }
        fn clear_all_api_keys(&self) -> AppResult<()> {
            if self.echec_suppression {
                return Err(AppError::Provider("coffre indisponible".into()));
            }
            self.cles.lock().unwrap().clear();
            Ok(())
        }
    }

    struct RepoMemoire {
        store: Mutex<Option<AppSettings>>,
        derniere_verification: Mutex<Option<String>>,
    }

    impl SettingsRepository for RepoMemoire {
        fn get(&self) -> AppResult<AppSettings> {
            Ok(self.store.lock().unwrap().clone().unwrap_or_default())
        }
        fn upsert(&self, settings: &AppSettings) -> AppResult<AppSettings> {
            *self.store.lock().unwrap() = Some(settings.clone());
            Ok(settings.clone())
        }
        fn last_update_check(&self) -> AppResult<Option<String>> {
            Ok(self.derniere_verification.lock().unwrap().clone())
        }
        fn mark_update_check(&self, instant: &str) -> AppResult<()> {
            *self.derniere_verification.lock().unwrap() = Some(instant.to_owned());
            Ok(())
        }
    }

    fn service() -> SettingsService<RepoMemoire, CoffreMemoire> {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        SettingsService::new(
            RepoMemoire {
                store: Mutex::new(None),
                derniere_verification: Mutex::new(None),
            },
            CoffreMemoire::default(),
            pool,
            PathBuf::new(),
        )
    }

    fn form(llm: LlmForm) -> Settings {
        Settings {
            llm,
            llm_presets: Default::default(),
            ai_routes: Default::default(),
            remote_send_consents: Default::default(),
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

    /// Le délai de vérification, décidé sans réseau ni horloge réelle.
    mod verification_de_mise_a_jour {
        use super::*;

        fn t(iso: &str) -> chrono::DateTime<chrono::Utc> {
            chrono::DateTime::parse_from_rfc3339(iso)
                .unwrap()
                .with_timezone(&chrono::Utc)
        }

        #[test]
        fn un_premier_lancement_verifie() {
            assert!(verification_maj_due(None, t("2026-10-05T12:00:00Z")));
        }

        #[test]
        fn rien_avant_vingt_quatre_heures() {
            let maintenant = t("2026-10-05T12:00:00Z");
            assert!(!verification_maj_due(
                Some("2026-10-05T11:59:00Z"),
                maintenant
            ));
            assert!(!verification_maj_due(
                Some("2026-10-04T12:00:01Z"),
                maintenant
            ));
        }

        #[test]
        fn vingt_quatre_heures_pile_suffisent() {
            let maintenant = t("2026-10-05T12:00:00Z");
            assert!(verification_maj_due(
                Some("2026-10-04T12:00:00Z"),
                maintenant
            ));
            assert!(verification_maj_due(
                Some("2026-10-03T08:00:00Z"),
                maintenant
            ));
        }

        #[test]
        fn un_horodatage_illisible_ne_bloque_pas_a_jamais() {
            assert!(verification_maj_due(
                Some("hier"),
                t("2026-10-05T12:00:00Z")
            ));
            assert!(verification_maj_due(Some(""), t("2026-10-05T12:00:00Z")));
        }

        #[test]
        fn un_horodatage_dans_le_futur_vient_d_une_horloge_dereglee() {
            // Sans ce cas, une horloge avancée d'un an puis remise à l'heure aurait suspendu
            // toute vérification pendant un an.
            assert!(verification_maj_due(
                Some("2027-10-05T12:00:00Z"),
                t("2026-10-05T12:00:00Z")
            ));
        }

        #[test]
        fn le_fuseau_de_l_horodatage_est_respecte() {
            // 14:30+02:00 vaut 12:30 UTC : deux heures plus tôt, pas vingt-deux.
            assert!(!verification_maj_due(
                Some("2026-10-05T14:30:00+02:00"),
                t("2026-10-05T14:30:00Z")
            ));
        }

        #[tokio::test]
        async fn un_appel_trop_tot_n_interroge_pas_github_et_laisse_l_horodatage() {
            let service = service();
            let pose = chrono::Utc::now().to_rfc3339();
            service.repo.mark_update_check(&pose).unwrap();

            // L'horodatage intact est la preuve qu'aucun appel n'a été tenté : il est
            // réécrit avant chaque interrogation de GitHub. Ce test ne touche donc pas le
            // réseau, qu'il soit joignable ou non.
            assert!(service.check_update_if_due().await.unwrap().is_none());
            assert_eq!(service.repo.last_update_check().unwrap(), Some(pose));
        }
    }

    #[test]
    fn les_consentements_d_envoi_distant_sont_persistes() {
        let service = service();
        let mut settings = form(ollama());
        settings.remote_send_consents.insert("claude".into());

        service.save(settings, None).unwrap();

        let relu = service.load().unwrap();
        assert!(relu.remote_send_consents.contains("claude"));
    }

    #[test]
    fn un_consentement_pour_l_ia_locale_ou_un_inconnu_est_refuse() {
        for id in ["candilog_local", "fournisseur-inconnu"] {
            let service = service();
            let mut settings = form(ollama());
            settings.remote_send_consents.insert(id.into());

            let error = service.save(settings, None).unwrap_err();

            assert!(matches!(error, AppError::Validation(_)), "{id} : {error:?}");
        }
    }

    #[test]
    fn les_routes_des_taches_sont_persistees() {
        let service = service();
        let mut settings = form(ollama());
        settings.ai_routes.insert(
            AiTask::AnalyzeResume,
            Some(TaskRoute {
                provider: ProviderKind::Ollama,
                model: "qwen2.5:7b".into(),
            }),
        );
        settings.ai_routes.insert(AiTask::ExtractOffer, None);

        let enregistre = service.save(settings, None).unwrap();
        assert_eq!(enregistre.ai_routes.len(), 2);
        let relu = service.load().unwrap();
        assert_eq!(relu.ai_routes.get(&AiTask::ExtractOffer), Some(&None));
        assert_eq!(
            relu.ai_routes
                .get(&AiTask::AnalyzeResume)
                .cloned()
                .flatten()
                .map(|route| route.model),
            Some("qwen2.5:7b".into())
        );
    }

    #[test]
    fn une_route_sans_modele_est_refusee() {
        let mut settings = form(ollama());
        settings.ai_routes.insert(
            AiTask::WriteLetter,
            Some(TaskRoute {
                provider: ProviderKind::Ollama,
                model: "  ".into(),
            }),
        );
        assert!(matches!(
            service().save(settings, None),
            Err(AppError::Validation(_))
        ));
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
            service
                .secret_store
                .load_api_key("openai")
                .unwrap()
                .as_deref(),
            Some("sk-test")
        );
    }

    /// Enregistrer un second fournisseur conserve le modèle, l'endpoint et la clé du premier.
    #[test]
    fn enregistrer_un_autre_fournisseur_n_ecrase_pas_le_precedent() {
        let service = service();
        let mut openai = ollama();
        openai.provider = ProviderKind::OpenAI;
        openai.endpoint = Some("https://api.openai.com".into());
        openai.model = "gpt-4o".into();
        openai.temperature = 0.2;
        service
            .save(form(openai), Some("sk-openai".into()))
            .unwrap();

        let mut mistral = ollama();
        mistral.provider = ProviderKind::Mistral;
        mistral.endpoint = Some("https://api.mistral.ai".into());
        mistral.model = "mistral-small".into();
        mistral.temperature = 0.9;
        let saved = service
            .save(form(mistral), Some("sk-mistral".into()))
            .unwrap();

        assert_eq!(saved.llm.provider, ProviderKind::Mistral);
        assert_eq!(saved.llm.model, "mistral-small");
        let openai_preset = saved.llm_presets.get("openai").expect("preset openai");
        assert_eq!(openai_preset.model, "gpt-4o");
        assert_eq!(
            openai_preset.endpoint.as_deref(),
            Some("https://api.openai.com")
        );
        assert!((openai_preset.temperature - 0.2).abs() < f32::EPSILON);
        assert!(openai_preset.api_key_configured);
        assert_eq!(
            service
                .secret_store
                .load_api_key("openai")
                .unwrap()
                .as_deref(),
            Some("sk-openai")
        );
        assert_eq!(
            service
                .secret_store
                .load_api_key("mistral")
                .unwrap()
                .as_deref(),
            Some("sk-mistral")
        );
    }

    /// Le formulaire décrit un fournisseur que l'utilisateur n'a pas encore enregistré : la
    /// clé du coffre appartient au couple (fournisseur, endpoint) mémorisé, et l'attacher à
    /// une adresse arbitraire reviendrait à la présenter en `Authorization` à un tiers.
    #[test]
    fn un_endpoint_non_persiste_ne_recoit_pas_la_cle_du_coffre() {
        let service = service();
        let mut stored = AppSettings::default();
        stored.llm.provider = ProviderKind::OpenAI;
        stored.llm.endpoint = Some("https://api.openai.com".into());
        stored.llm.model = "gpt-4o".into();
        stored.capture_llm_preset();
        service.repo.upsert(&stored).unwrap();
        service
            .secret_store
            .store_api_key("openai", Some("sk-test"))
            .unwrap();

        let mut llm = ollama();
        llm.provider = ProviderKind::Custom("maison".into());
        llm.endpoint = Some("https://exfiltration.example".into());
        llm.model = "gpt-4o".into();

        assert!(matches!(
            service.provider_config(llm, None),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn ollama_ne_lit_pas_le_coffre() {
        let service = service();
        service
            .secret_store
            .store_api_key("openai", Some("sk-cachee"))
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
        stored.capture_llm_preset();
        service.repo.upsert(&stored).unwrap();
        service
            .secret_store
            .store_api_key("openai", Some("sk-secret"))
            .unwrap();

        let payload = service.load().unwrap();
        let json = serde_json::to_string(&payload).unwrap();

        assert!(payload.llm.api_key_configured);
        assert!(!json.contains("sk-secret"));
        assert!(!json.contains("api_key\":"));
    }

    /// La clé du coffre est reprise sans ressaisie — pour le fournisseur et l'endpoint
    /// mémorisés, y compris via un preset non actif.
    #[test]
    fn provider_config_charge_le_secret_du_coffre() {
        let service = service();
        let mut stored = AppSettings::default();
        stored.llm.provider = ProviderKind::OpenAI;
        stored.llm.endpoint = Some("https://api.openai.com".into());
        stored.llm.model = "gpt-4o".into();
        stored.capture_llm_preset();
        service.repo.upsert(&stored).unwrap();
        service
            .secret_store
            .store_api_key("openai", Some("sk-stored"))
            .unwrap();
        let mut llm = ollama();
        llm.provider = ProviderKind::OpenAI;
        llm.endpoint = Some("https://api.openai.com".into());
        llm.model = "gpt-4o".into();

        let config = service.provider_config(llm, None).unwrap();

        assert_eq!(config.api_key.as_deref(), Some("sk-stored"));
    }

    #[test]
    fn provider_config_reutilise_la_cle_d_un_preset_inactif() {
        let service = service();
        let mut openai = ollama();
        openai.provider = ProviderKind::OpenAI;
        openai.endpoint = Some("https://api.openai.com".into());
        openai.model = "gpt-4o".into();
        service
            .save(form(openai), Some("sk-openai".into()))
            .unwrap();
        let mut mistral = ollama();
        mistral.provider = ProviderKind::Mistral;
        mistral.endpoint = Some("https://api.mistral.ai".into());
        mistral.model = "mistral-small".into();
        service
            .save(form(mistral), Some("sk-mistral".into()))
            .unwrap();

        let mut llm = ollama();
        llm.provider = ProviderKind::OpenAI;
        llm.endpoint = Some("https://api.openai.com".into());
        llm.model = "gpt-4o".into();
        let config = service.provider_config(llm, None).unwrap();
        assert_eq!(config.api_key.as_deref(), Some("sk-openai"));
    }

    #[test]
    fn provider_config_prefere_la_nouvelle_cle() {
        let service = service();
        service
            .secret_store
            .store_api_key("openai", Some("sk-stored"))
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
    fn clear_api_key_supprime_uniquement_le_fournisseur_actif() {
        let service = service();
        let mut openai = ollama();
        openai.provider = ProviderKind::OpenAI;
        openai.model = "gpt-4o".into();
        service
            .save(form(openai), Some("sk-openai".into()))
            .unwrap();
        let mut mistral = ollama();
        mistral.provider = ProviderKind::Mistral;
        mistral.model = "mistral-small".into();
        service
            .save(form(mistral), Some("sk-mistral".into()))
            .unwrap();

        service.clear_api_key().unwrap();

        assert_eq!(service.secret_store.load_api_key("mistral").unwrap(), None);
        assert_eq!(
            service
                .secret_store
                .load_api_key("openai")
                .unwrap()
                .as_deref(),
            Some("sk-openai")
        );
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
            .store_api_key("openai", Some("sk-stored"))
            .unwrap();

        let outcome = service.reset().unwrap();

        let remaining: i64 = connection(&service.pool)
            .unwrap()
            .query_row("SELECT COUNT(*) FROM app_kv", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 0);
        assert_eq!(service.secret_store.load_api_key("openai").unwrap(), None);
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
                derniere_verification: Mutex::new(None),
            },
            CoffreMemoire {
                cles: Mutex::new(HashMap::from([("openai".into(), "sk-stored".into())])),
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
