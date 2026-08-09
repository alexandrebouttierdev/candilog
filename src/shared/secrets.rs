//! Stockage des secrets applicatifs dans le coffre natif du système.

use crate::shared::error::{AppError, AppResult};

/// Accès à la clé API du fournisseur IA, rangée dans le coffre natif du système.
///
/// L'entrée est fixe : l'application est mono-utilisateur depuis le retrait de
/// l'authentification. Une clé enregistrée par une version antérieure vivait sous
/// `llm-api-key:{user_id}` et n'est pas récupérable — l'utilisateur la ressaisit une fois.
/// Aucune migration n'est possible : au démarrage l'application n'a plus de session, donc
/// plus rien qui permette de retrouver l'ancien identifiant.
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
