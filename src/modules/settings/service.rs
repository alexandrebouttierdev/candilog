//! Logique métier des paramètres applicatifs.

use crate::modules::settings::model::AppSettings;
use crate::modules::settings::repository::SettingsRepository;
use crate::shared::error::{AppError, AppResult};
use crate::shared::llm::ProviderKind;

/// Service métier des paramètres, générique sur le dépôt (testable via mock).
pub struct SettingsService<R: SettingsRepository> {
    repo: R,
}

impl<R: SettingsRepository> SettingsService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Récupère les paramètres applicatifs (valeurs par défaut si aucune n'a encore été
    /// enregistrée).
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn get(&self) -> AppResult<AppSettings> {
        self.repo.get()
    }

    /// Valide puis persiste les paramètres.
    ///
    /// # Errors
    /// `AppError::Validation` si la configuration est invalide ; sinon l'erreur du dépôt.
    pub fn update(&self, settings: &AppSettings) -> AppResult<AppSettings> {
        validate(settings)?;
        self.repo.upsert(settings)
    }

    /// Valide les paramètres sans les persister.
    ///
    /// # Errors
    /// Retourne `Validation` si la configuration est incohérente.
    pub fn validate(settings: &AppSettings) -> AppResult<()> {
        validate(settings)
    }

    /// Persiste des paramètres déjà validés par l'appelant.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn persist(&self, settings: &AppSettings) -> AppResult<AppSettings> {
        self.repo.upsert(settings)
    }
}

/// Valide une configuration applicative.
fn validate(settings: &AppSettings) -> AppResult<()> {
    let llm = &settings.llm;
    if !(0.0..=2.0).contains(&llm.temperature) {
        return Err(AppError::Validation(
            "La température doit être comprise entre 0.0 et 2.0".into(),
        ));
    }
    match &llm.provider {
        ProviderKind::Ollama => {}
        ProviderKind::Custom(_) => {
            if llm
                .endpoint
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                return Err(AppError::Validation(
                    "Un endpoint est requis pour un fournisseur personnalisé".into(),
                ));
            }
        }
        ProviderKind::Claude
        | ProviderKind::OpenAI
        | ProviderKind::Gemini
        | ProviderKind::Mistral
        | ProviderKind::Nvidia => {
            if llm.api_key.as_deref().unwrap_or_default().trim().is_empty() {
                return Err(AppError::Validation(
                    "Une clé API est requise pour ce fournisseur".into(),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
