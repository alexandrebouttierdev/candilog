//! Cas d'usage du référentiel des secteurs.

use crate::core::errors::AppResult;
use crate::features::sectors::domain::{ActivitySector, SectorRepository};

/// Service du référentiel, générique sur le dépôt.
pub struct SectorService<R: SectorRepository> {
    repo: R,
}

impl<R: SectorRepository> SectorService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// List les secteurs dans l'ordre d'affichage.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list(&self) -> AppResult<Vec<ActivitySector>> {
        self.repo.list()
    }
}
