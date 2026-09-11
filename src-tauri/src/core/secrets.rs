//! Coffre natif du système pour les clés API des fournisseurs IA.

use crate::core::config::APP_IDENTIFIER;
use crate::core::errors::{AppError, AppResult};

/// Préfixe des entrées de trousseau : une clé par fournisseur (`llm-api-key-openai`, …).
const KEY_PREFIX: &str = "llm-api-key-";

/// Ancienne entrée unique (avant la séparation par fournisseur).
const LEGACY_KEY_NAME: &str = "llm-api-key";

/// Identifiants de fournisseurs pour lesquels une clé peut exister dans le coffre.
const PROVIDER_IDS: &[&str] = &[
    "candilog_local",
    "ollama",
    "claude",
    "openai",
    "gemini",
    "mistral",
    "deepseek",
    "custom",
];

/// Service de trousseau des versions antérieures à l'unification de l'identifiant.
///
/// Aucune version publique ne l'a utilisé ; il ne subsiste que sur les postes de
/// développement, d'où la reprise silencieuse au premier accès.
const LEGACY_SERVICE: &str = "com.alexandrebouttier.candilog";

/// Accès aux clés API, rangées hors de SQLite.
///
/// Chaque fournisseur a sa propre entrée : basculer d'OpenAI à Mistral ne doit jamais
/// écraser la clé de l'autre. Une clé héritée encore présente dans le JSON `parametres`
/// est déplacée vers le coffre au chargement.
#[derive(Debug, Clone, Default)]
pub struct SecretStore;

/// Message unique pour un trousseau système hors d'atteinte.
fn unavailable_store() -> AppError {
    AppError::Provider(
        "Le trousseau du système est indisponible : Candilog ne peut pas y lire ni y écrire \
         la clé du fournisseur IA."
            .into(),
    )
}

/// Message unique pour une lecture refusée par le trousseau.
fn read_error(error: &keyring::Error) -> AppError {
    tracing::error!(%error, "lecture du coffre de secrets impossible");
    AppError::Provider(
        "La clé du fournisseur IA n'a pas pu être lue dans le trousseau du système.".into(),
    )
}

impl SecretStore {
    fn entry_for(service: &str, key_name: &str) -> AppResult<keyring::Entry> {
        keyring::Entry::new(service, key_name).map_err(|error| {
            // L'erreur du trousseau est en anglais et nomme le service système : elle part
            // au journal, jamais à l'écran (`docs/CODE_RULES.md` §1, §13).
            tracing::error!(%error, "coffre de secrets indisponible");
            unavailable_store()
        })
    }

    fn entry(provider_id: &str) -> AppResult<keyring::Entry> {
        Self::entry_for(APP_IDENTIFIER, &format!("{KEY_PREFIX}{provider_id}"))
    }

    fn legacy_entry() -> AppResult<keyring::Entry> {
        Self::entry_for(APP_IDENTIFIER, LEGACY_KEY_NAME)
    }

    /// Déplace la clé de l'ancien service de trousseau vers le nouveau, une seule fois.
    fn reprendre_entree_heritee(provider_id: &str) {
        let Ok(ancienne) = Self::entry_for(LEGACY_SERVICE, LEGACY_KEY_NAME) else {
            return;
        };
        let Ok(secret) = ancienne.get_password() else {
            return;
        };
        match Self::entry(provider_id).and_then(|entree| {
            entree
                .set_password(&secret)
                .map_err(|e| AppError::Provider(e.to_string()))
        }) {
            Ok(()) => {
                let _ = ancienne.delete_credential();
                tracing::info!(
                    provider_id,
                    "clé du fournisseur IA reprise depuis l'ancien trousseau"
                );
            }
            Err(error) => tracing::warn!(%error, "reprise de la clé héritée impossible"),
        }
    }

    /// Migre l'ancienne entrée unique `llm-api-key` vers `llm-api-key-{provider_id}`.
    fn migrer_cle_unique(provider_id: &str) -> AppResult<()> {
        let legacy = Self::legacy_entry()?;
        let secret = match legacy.get_password() {
            Ok(secret) => secret,
            Err(keyring::Error::NoEntry) => return Ok(()),
            Err(error) => return Err(read_error(&error)),
        };
        let cible = Self::entry(provider_id)?;
        match cible.get_password() {
            Ok(_) => {
                // Une clé spécifique existe déjà : on retire seulement l'ancienne.
                let _ = legacy.delete_credential();
                Ok(())
            }
            Err(keyring::Error::NoEntry) => {
                cible.set_password(&secret).map_err(|error| {
                    tracing::error!(%error, "migration de la clé héritée impossible");
                    AppError::Provider(
                        "La clé n'a pas pu être enregistrée dans le trousseau du système.".into(),
                    )
                })?;
                let _ = legacy.delete_credential();
                tracing::info!(
                    provider_id,
                    "clé IA unique migrée vers l'entrée du fournisseur actif"
                );
                Ok(())
            }
            Err(error) => Err(read_error(&error)),
        }
    }

    /// Lit la clé du fournisseur indiqué.
    ///
    /// # Errors
    /// Retourne une erreur si le coffre système est inaccessible.
    pub fn load_api_key(&self, provider_id: &str) -> AppResult<Option<String>> {
        Self::migrer_cle_unique(provider_id)?;
        match Self::entry(provider_id)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => {
                Self::reprendre_entree_heritee(provider_id);
                match Self::entry(provider_id)?.get_password() {
                    Ok(secret) => Ok(Some(secret)),
                    Err(keyring::Error::NoEntry) => Ok(None),
                    Err(error) => Err(read_error(&error)),
                }
            }
            Err(error) => Err(read_error(&error)),
        }
    }

    /// Enregistre ou supprime la clé du fournisseur indiqué.
    ///
    /// # Errors
    /// Retourne une erreur si le coffre système est inaccessible.
    pub fn store_api_key(&self, provider_id: &str, secret: Option<&str>) -> AppResult<()> {
        let entry = Self::entry(provider_id)?;
        match secret.filter(|value| !value.trim().is_empty()) {
            Some(secret) => entry.set_password(secret).map_err(|error| {
                tracing::error!(%error, "écriture dans le coffre de secrets impossible");
                AppError::Provider(
                    "La clé n'a pas pu être enregistrée dans le trousseau du système.".into(),
                )
            }),
            None => match entry.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                Err(error) => {
                    tracing::error!(%error, "suppression du secret impossible");
                    Err(AppError::Provider(
                        "La clé n'a pas pu être retirée du trousseau du système.".into(),
                    ))
                }
            },
        }
    }

    /// Supprime toutes les clés fournisseurs connues (réinitialisation).
    ///
    /// # Errors
    /// Retourne une erreur si une suppression échoue pour une raison autre que l'absence d'entrée.
    pub fn clear_all_api_keys(&self) -> AppResult<()> {
        for provider_id in PROVIDER_IDS {
            self.store_api_key(provider_id, None)?;
        }
        match Self::legacy_entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => {
                tracing::error!(%error, "suppression de la clé héritée impossible");
                Err(AppError::Provider(
                    "La clé n'a pas pu être retirée du trousseau du système.".into(),
                ))
            }
        }
    }
}

/// Contract testable du coffre, pour ne pas dépendre du trousseau en tests unitaires.
pub trait SecretStoreContract: Send + Sync {
    fn load_api_key(&self, provider_id: &str) -> AppResult<Option<String>>;
    fn store_api_key(&self, provider_id: &str, secret: Option<&str>) -> AppResult<()>;
    fn clear_all_api_keys(&self) -> AppResult<()>;
}

impl SecretStoreContract for SecretStore {
    fn load_api_key(&self, provider_id: &str) -> AppResult<Option<String>> {
        Self::load_api_key(self, provider_id)
    }

    fn store_api_key(&self, provider_id: &str, secret: Option<&str>) -> AppResult<()> {
        Self::store_api_key(self, provider_id, secret)
    }

    fn clear_all_api_keys(&self) -> AppResult<()> {
        Self::clear_all_api_keys(self)
    }
}
