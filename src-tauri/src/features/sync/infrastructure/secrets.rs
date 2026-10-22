//! Secrets de la synchronisation dans le coffre du système (`keyring`) : jetons OAuth.
//! Jamais dans SQLite, jamais dans les journaux.

use crate::core::config::APP_IDENTIFIER;
use crate::core::errors::{AppError, AppResult};
use zeroize::Zeroizing;

/// Accès aux secrets de synchronisation.
pub trait SyncSecrets: Send + Sync {
    /// # Errors
    /// `Sync` si le coffre est inaccessible.
    fn load(&self, name: &str) -> AppResult<Option<Zeroizing<String>>>;
    /// # Errors
    /// `Sync` si le coffre est inaccessible.
    fn store(&self, name: &str, value: &str) -> AppResult<()>;
    /// # Errors
    /// `Sync` si le coffre est inaccessible.
    fn delete(&self, name: &str) -> AppResult<()>;
}

/// Entrée historique de clé, uniquement supprimée lors de la déconnexion d'anciens espaces.
#[must_use]
pub fn space_entry(space_id: &str) -> String {
    format!("sync-space-{space_id}")
}

/// Entrée du jeton de renouvellement d'un fournisseur.
#[must_use]
pub fn token_entry(provider: &str) -> String {
    format!("sync-oauth-{provider}")
}

fn unavailable(error: &keyring::Error) -> AppError {
    tracing::error!(%error, "coffre système inaccessible pour la synchronisation");
    AppError::Sync(
        "Le trousseau du système est indisponible : Candilog ne peut pas y conserver les \
         jetons du fournisseur."
            .into(),
    )
}

/// Coffre natif (Trousseau macOS, Gestionnaire d'identification Windows, Secret Service).
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyringSyncSecrets;

impl SyncSecrets for KeyringSyncSecrets {
    fn load(&self, name: &str) -> AppResult<Option<Zeroizing<String>>> {
        let entry = keyring::Entry::new(APP_IDENTIFIER, name).map_err(|e| unavailable(&e))?;
        match entry.get_password() {
            Ok(secret) => Ok(Some(Zeroizing::new(secret))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(unavailable(&error)),
        }
    }

    fn store(&self, name: &str, value: &str) -> AppResult<()> {
        let entry = keyring::Entry::new(APP_IDENTIFIER, name).map_err(|e| unavailable(&e))?;
        entry
            .set_password(value)
            .map_err(|error| unavailable(&error))
    }

    fn delete(&self, name: &str) -> AppResult<()> {
        let entry = keyring::Entry::new(APP_IDENTIFIER, name).map_err(|e| unavailable(&e))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(unavailable(&error)),
        }
    }
}

/// Coffre en mémoire, pour les tests.
#[cfg(test)]
#[derive(Debug, Default)]
pub struct MemorySecrets(pub std::sync::Mutex<std::collections::BTreeMap<String, String>>);

#[cfg(test)]
impl SyncSecrets for MemorySecrets {
    fn load(&self, name: &str) -> AppResult<Option<Zeroizing<String>>> {
        Ok(self
            .0
            .lock()
            .map_err(|_| AppError::Sync("verrou".into()))?
            .get(name)
            .cloned()
            .map(Zeroizing::new))
    }

    fn store(&self, name: &str, value: &str) -> AppResult<()> {
        self.0
            .lock()
            .map_err(|_| AppError::Sync("verrou".into()))?
            .insert(name.to_owned(), value.to_owned());
        Ok(())
    }

    fn delete(&self, name: &str) -> AppResult<()> {
        self.0
            .lock()
            .map_err(|_| AppError::Sync("verrou".into()))?
            .remove(name);
        Ok(())
    }
}
