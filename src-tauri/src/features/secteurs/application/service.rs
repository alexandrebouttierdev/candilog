//! Cas d'usage du référentiel des secteurs.

use crate::core::errors::AppResult;
use crate::features::secteurs::domain::{SecteurActivite, SecteurRepository};

/// Service du référentiel, générique sur le dépôt.
pub struct SecteurService<R: SecteurRepository> {
    repo: R,
}

impl<R: SecteurRepository> SecteurService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Liste les secteurs dans l'ordre d'affichage.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn lister(&self) -> AppResult<Vec<SecteurActivite>> {
        self.repo.lister()
    }
}
