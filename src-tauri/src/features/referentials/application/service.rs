//! Cas d'usage des référentiels métier.

use crate::core::errors::AppResult;
use crate::features::referentials::domain::{ReferentialRepository, Referentials};

/// Service des référentiels, générique sur le dépôt.
pub struct ReferentialService<R: ReferentialRepository> {
    repo: R,
}

impl<R: ReferentialRepository> ReferentialService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Charge les quatre catalogues dans leur ordre d'affichage.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn load(&self) -> AppResult<Referentials> {
        self.repo.load()
    }
}
