//! Coffre natif du système pour la clé API du fournisseur IA.

use crate::core::config::APP_IDENTIFIER;
use crate::core::errors::{AppError, AppResult};

/// Nom de l'entrée de trousseau portant la clé du fournisseur IA.
const KEY_NAME: &str = "llm-api-key";

/// Service de trousseau des versions antérieures à l'unification de l'identifiant.
///
/// Aucune version publique ne l'a utilisé ; il ne subsiste que sur les postes de
/// développement, d'où la reprise silencieuse au premier accès.
const LEGACY_SERVICE: &str = "com.alexandrebouttier.candilog";

/// Accès à la clé API, rangée hors de SQLite.
///
/// L'entrée est fixe : l'application est mono-utilisateur. Une clé héritée encore présente
/// dans le JSON `parametres` est déplacée vers le coffre au chargement.
#[derive(Debug, Clone, Default)]
pub struct SecretStore;

impl SecretStore {
    fn entry_for(service: &str) -> AppResult<keyring::Entry> {
        keyring::Entry::new(service, KEY_NAME)
            .map_err(|e| AppError::Provider(format!("Coffre de secrets indisponible : {e}")))
    }

    fn entry() -> AppResult<keyring::Entry> {
        Self::entry_for(APP_IDENTIFIER)
    }

    /// Déplace la clé de l'ancien service de trousseau vers le nouveau, une seule fois.
    ///
    /// Ne s'exécute que si le nouveau service n'a pas d'entrée. L'ancienne n'est supprimée
    /// qu'après une écriture réussie : une coupure entre les deux laisse la clé lisible à
    /// son emplacement d'origine plutôt que nulle part.
    fn reprendre_entree_heritee() {
        let Ok(ancienne) = Self::entry_for(LEGACY_SERVICE) else {
            return;
        };
        let Ok(secret) = ancienne.get_password() else {
            return;
        };
        match Self::entry().and_then(|entree| {
            entree
                .set_password(&secret)
                .map_err(|e| AppError::Provider(e.to_string()))
        }) {
            Ok(()) => {
                let _ = ancienne.delete_credential();
                tracing::info!("clé du fournisseur IA reprise depuis l'ancien trousseau");
            }
            Err(error) => tracing::warn!(%error, "reprise de la clé héritée impossible"),
        }
    }

    /// Lit la clé du fournisseur IA.
    ///
    /// # Errors
    /// Retourne une erreur si le coffre système est inaccessible.
    pub fn load_api_key(&self) -> AppResult<Option<String>> {
        match Self::entry()?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => {
                Self::reprendre_entree_heritee();
                match Self::entry()?.get_password() {
                    Ok(secret) => Ok(Some(secret)),
                    Err(keyring::Error::NoEntry) => Ok(None),
                    Err(e) => Err(AppError::Provider(format!(
                        "Lecture du coffre de secrets impossible : {e}"
                    ))),
                }
            }
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

/// Contract testable du coffre, pour ne pas dépendre du trousseau en tests unitaires.
pub trait SecretStoreContract: Send + Sync {
    fn load_api_key(&self) -> AppResult<Option<String>>;
    fn store_api_key(&self, secret: Option<&str>) -> AppResult<()>;
}

impl SecretStoreContract for SecretStore {
    fn load_api_key(&self) -> AppResult<Option<String>> {
        Self::load_api_key(self)
    }

    fn store_api_key(&self, secret: Option<&str>) -> AppResult<()> {
        Self::store_api_key(self, secret)
    }
}
