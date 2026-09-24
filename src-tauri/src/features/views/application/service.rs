//! Validation et orchestration des vues enregistrées.

use crate::core::errors::{AppError, AppResult};
use crate::features::views::domain::{NewSavedView, SavedView, SavedViewRepository, MAX_VIEW_NAME};
use uuid::Uuid;

/// Service des vues : l'entrée IPC est revalidée ici, jamais crue sur parole.
pub struct SavedViewService<R: SavedViewRepository> {
    repo: R,
}

impl<R: SavedViewRepository> SavedViewService<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list(&self) -> AppResult<Vec<SavedView>> {
        self.repo.list()
    }

    /// # Errors
    /// `AppError::Validation` si le nom est vide ou trop long.
    pub fn create(&self, input: &NewSavedView) -> AppResult<SavedView> {
        validate(input)?;
        self.repo.create(input)
    }

    /// # Errors
    /// `AppError::Validation` si le nom est vide ou trop long ; `NotFound` si la vue n'existe pas.
    pub fn update(&self, id: Uuid, input: &NewSavedView) -> AppResult<SavedView> {
        validate(input)?;
        self.repo.update(id, input)
    }

    /// Copie une vue sous un nom « (copie) », en dernière position.
    ///
    /// # Errors
    /// `NotFound` si la vue n'existe pas.
    pub fn duplicate(&self, id: Uuid) -> AppResult<SavedView> {
        let source = self.repo.get(id)?;
        let suffix = " (copie)";
        let base: String = source
            .name
            .chars()
            .take(MAX_VIEW_NAME - suffix.chars().count())
            .collect();
        self.repo.create(&NewSavedView {
            name: format!("{}{suffix}", base.trim_end()),
            filter: source.filter,
        })
    }

    /// # Errors
    /// `NotFound` si la vue n'existe pas.
    pub fn delete(&self, id: Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }
}

fn validate(input: &NewSavedView) -> AppResult<()> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("Donnez un nom à la vue.".into()));
    }
    if name.chars().count() > MAX_VIEW_NAME {
        return Err(AppError::Validation(format!(
            "Le nom d'une vue tient en {MAX_VIEW_NAME} caractères."
        )));
    }
    Ok(())
}
