//! Logique métier du référentiel des secteurs d'activité.

use crate::modules::secteurs::model::SecteurActivite;
use crate::modules::secteurs::repository::SecteurRepository;
use crate::shared::error::AppResult;

/// Service du référentiel des secteurs, générique sur le dépôt (testable via mock).
pub struct SecteurService<R: SecteurRepository> {
    repo: R,
}

impl<R: SecteurRepository> SecteurService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub fn new(repo: R) -> Self {
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

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
