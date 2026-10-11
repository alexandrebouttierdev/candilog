//! Contrat d'un fournisseur de stockage distant.
//!
//! Le moteur ne connaît que ces quatre opérations sur des objets nommés : un dossier
//! local, Google Drive et OneDrive les implémentent chacun à sa façon. Ajouter un
//! fournisseur revient à écrire un nouvel adaptateur, sans toucher au protocole.

use async_trait::async_trait;
use std::time::Duration;

/// Objet présent dans l'espace distant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteObject {
    pub name: String,
    pub size: u64,
}

/// Résultat d'une écriture « création seule ».
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PutOutcome {
    Created,
    /// Un objet portait déjà ce nom : il n'a pas été remplacé.
    AlreadyExists,
}

/// Échec d'une opération distante, classé selon la réaction attendue du moteur.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RemoteError {
    /// Réseau absent, délai dépassé : réessayer plus tard, sans alerte.
    #[error("hors ligne : {0}")]
    Offline(String),
    /// Limite de débit du fournisseur : attendre le délai indiqué.
    #[error("limite de débit atteinte")]
    RateLimited(Option<Duration>),
    /// Session expirée ou révoquée : l'utilisateur doit se reconnecter.
    #[error("authentification requise : {0}")]
    Unauthorized(String),
    /// Espace, dossier ou fichier inaccessible (dossier démonté, droits retirés).
    #[error("stockage inaccessible : {0}")]
    Unavailable(String),
    /// Réponse inattendue ou erreur côté fournisseur : réessayer avec un délai croissant.
    #[error("erreur du fournisseur : {0}")]
    Provider(String),
}

impl RemoteError {
    /// Erreur passagère, qui justifie une nouvelle tentative automatique.
    #[must_use]
    pub const fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::Offline(_) | Self::RateLimited(_) | Self::Provider(_) | Self::Unavailable(_)
        )
    }
}

/// Stockage distant d'objets immuables.
///
/// Les noms sont des chemins relatifs à l'espace (`changes/<installation>/<seq>.batch`).
/// Aucune implémentation ne doit suivre un nom vers l'extérieur de l'espace : les noms
/// sont produits par le moteur, mais un adaptateur les revalide malgré tout.
#[async_trait]
pub trait RemoteStore: Send + Sync {
    /// Objets dont le nom commence par `prefix`.
    async fn list(&self, prefix: &str) -> Result<Vec<RemoteObject>, RemoteError>;

    /// Contenu d'un objet, `None` s'il n'existe pas.
    async fn get(&self, name: &str) -> Result<Option<Vec<u8>>, RemoteError>;

    /// Crée un objet ; ne remplace jamais un objet existant.
    async fn put_new(&self, name: &str, bytes: &[u8]) -> Result<PutOutcome, RemoteError>;

    /// Supprime un objet ; absent n'est pas une erreur.
    async fn delete(&self, name: &str) -> Result<(), RemoteError>;
}
