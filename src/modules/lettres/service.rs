//! Validation métier de la bibliothèque de lettres.

use crate::modules::lettres::dtos::NouvelleLettre;
use crate::modules::lettres::model::LettreMotivation;
use crate::modules::lettres::repository::LettreRepository;
use crate::shared::error::{AppError, AppResult};

/// Service des lettres, générique pour rester testable.
pub struct LettreService<R: LettreRepository> {
    repo: R,
}

impl<R: LettreRepository> LettreService<R> {
    #[must_use]
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub fn save(&self, input: &NouvelleLettre) -> AppResult<LettreMotivation> {
        if input.name.trim().is_empty() {
            return Err(AppError::Validation(
                "Le nom de la lettre est requis".into(),
            ));
        }
        if input.name.chars().count() > 140 {
            return Err(AppError::Validation(
                "Le nom de la lettre est trop long".into(),
            ));
        }
        if input.content.trim().is_empty() {
            return Err(AppError::Validation(
                "Générez une lettre avant de l'enregistrer".into(),
            ));
        }
        self.repo.create(input)
    }

    pub fn list(&self) -> AppResult<Vec<LettreMotivation>> {
        self.repo.list()
    }

    pub fn load(&self, id: uuid::Uuid) -> AppResult<LettreMotivation> {
        self.repo.get(id)
    }

    pub fn delete(&self, id: uuid::Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }
}
