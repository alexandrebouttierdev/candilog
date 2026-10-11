//! Fournisseur simulé en mémoire, pour les tests : partagé entre plusieurs « ordinateurs »,
//! il sait aussi simuler une coupure réseau et une écriture au résultat incertain.

use crate::features::sync::domain::remote::{PutOutcome, RemoteError, RemoteObject, RemoteStore};
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Espace distant simulé. Les clones partagent le même contenu.
#[derive(Debug, Clone, Default)]
pub struct MemoryStore {
    objects: Arc<Mutex<BTreeMap<String, Vec<u8>>>>,
    offline: Arc<AtomicBool>,
    /// Les N prochaines écritures réussissent côté fournisseur mais la réponse se perd.
    lost_acks: Arc<AtomicUsize>,
    pub puts: Arc<AtomicUsize>,
}

impl MemoryStore {
    pub fn set_offline(&self, offline: bool) {
        self.offline.store(offline, Ordering::SeqCst);
    }

    pub fn lose_next_acks(&self, count: usize) {
        self.lost_acks.store(count, Ordering::SeqCst);
    }

    pub fn names(&self) -> Vec<String> {
        self.objects
            .lock()
            .map(|objects| objects.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn raw(&self, name: &str) -> Option<Vec<u8>> {
        self.objects.lock().ok()?.get(name).cloned()
    }

    pub fn overwrite(&self, name: &str, bytes: Vec<u8>) {
        if let Ok(mut objects) = self.objects.lock() {
            objects.insert(name.to_owned(), bytes);
        }
    }

    pub fn remove(&self, name: &str) {
        if let Ok(mut objects) = self.objects.lock() {
            objects.remove(name);
        }
    }

    fn check(&self) -> Result<(), RemoteError> {
        if self.offline.load(Ordering::SeqCst) {
            Err(RemoteError::Offline("réseau simulé coupé".into()))
        } else {
            Ok(())
        }
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, BTreeMap<String, Vec<u8>>>, RemoteError> {
        self.objects
            .lock()
            .map_err(|_| RemoteError::Provider("verrou empoisonné".into()))
    }
}

#[async_trait]
impl RemoteStore for MemoryStore {
    async fn list(&self, prefix: &str) -> Result<Vec<RemoteObject>, RemoteError> {
        self.check()?;
        Ok(self
            .lock()?
            .iter()
            .filter(|(name, _)| name.starts_with(prefix))
            .map(|(name, bytes)| RemoteObject {
                name: name.clone(),
                size: bytes.len() as u64,
            })
            .collect())
    }

    async fn get(&self, name: &str) -> Result<Option<Vec<u8>>, RemoteError> {
        self.check()?;
        Ok(self.lock()?.get(name).cloned())
    }

    async fn put_new(&self, name: &str, bytes: &[u8]) -> Result<PutOutcome, RemoteError> {
        self.check()?;
        let mut objects = self.lock()?;
        if objects.contains_key(name) {
            return Ok(PutOutcome::AlreadyExists);
        }
        objects.insert(name.to_owned(), bytes.to_vec());
        self.puts.fetch_add(1, Ordering::SeqCst);
        let lost = self.lost_acks.load(Ordering::SeqCst);
        if lost > 0 {
            self.lost_acks.store(lost - 1, Ordering::SeqCst);
            return Err(RemoteError::Offline("réponse perdue après écriture".into()));
        }
        Ok(PutOutcome::Created)
    }

    async fn delete(&self, name: &str) -> Result<(), RemoteError> {
        self.check()?;
        self.lock()?.remove(name);
        Ok(())
    }
}
