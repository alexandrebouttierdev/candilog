//! Coffre natif du système pour la clé API du fournisseur IA.

use crate::core::errors::{AppError, AppResult};

/// Accès à la clé API, rangée hors de SQLite.
///
/// L'entrée est fixe : l'application est mono-utilisateur. Une clé héritée encore présente
/// dans le JSON `parametres` est déplacée vers le coffre au chargement.
#[derive(Debug, Clone, Default)]
pub struct SecretStore;

impl SecretStore {
    fn entry() -> AppResult<keyring::Entry> {
        keyring::Entry::new("com.alexandrebouttier.candilog", "llm-api-key")
            .map_err(|e| AppError::Provider(format!("Coffre de secrets indisponible : {e}")))
    }

    /// Lit la clé du fournisseur IA.
    ///
    /// # Errors
    /// Retourne une erreur si le coffre système est inaccessible.
    pub fn load_api_key(&self) -> AppResult<Option<String>> {
        match Self::entry()?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Provider(format!(
                "Lecture du coffre de secrets impossible : {e}"
            ))),
        }
    }

    /// Enregistre ou supprime la clé du fournisseur IA.
    ///
    /// # Errors
    /// Retourne une erreur si le coffre système est inaccessible.
    pub fn store_api_key(&self, secret: Option<&str>) -> AppResult<()> {
        let entry = Self::entry()?;
        match secret.filter(|value| !value.trim().is_empty()) {
            Some(secret) => entry.set_password(secret).map_err(|e| {
                AppError::Provider(format!(
                    "Écriture dans le coffre de secrets impossible : {e}"
                ))
            }),
            None => match entry.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                Err(e) => Err(AppError::Provider(format!(
                    "Suppression du secret impossible : {e}"
                ))),
            },
        }
    }
}

/// Contrat testable du coffre, pour ne pas dépendre du trousseau en tests unitaires.
pub trait CoffreSecrets: Send + Sync {
    fn load_api_key(&self) -> AppResult<Option<String>>;
    fn store_api_key(&self, secret: Option<&str>) -> AppResult<()>;
}

impl CoffreSecrets for SecretStore {
    fn load_api_key(&self) -> AppResult<Option<String>> {
        Self::load_api_key(self)
    }

    fn store_api_key(&self, secret: Option<&str>) -> AppResult<()> {
        Self::store_api_key(self, secret)
    }
}
