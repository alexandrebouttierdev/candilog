//! Cas d'usage des entretiens.

use crate::core::errors::{AppError, AppResult};
use crate::features::interviews::domain::{
    InterviewAnalysis, Interview, InterviewRepository, NewInterview,
};
use uuid::Uuid;

/// Service métier des entretiens, générique sur le dépôt.
pub struct InterviewService<R: InterviewRepository> {
    repo: R,
}

impl<R: InterviewRepository> InterviewService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// List tous les entretiens.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list(&self) -> AppResult<Vec<Interview>> {
        self.repo.list()
    }

    /// List les entretiens d'une plage de dates, bornes incluses.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list_between(&self, from: &str, to: &str) -> AppResult<Vec<Interview>> {
        self.repo.list_between(from, to)
    }

    /// Récupère un entretien par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn get(&self, id: Uuid) -> AppResult<Interview> {
        self.repo.get(id)
    }

    /// Valide puis enregistre l'entretien, en faisant avancer sa candidature.
    ///
    /// # Errors
    /// `AppError::Validation` si la candidature ou la date manque ;
    /// `AppError::NotFound` si `id` est fourni mais inconnu.
    pub fn save(&self, id: Option<Uuid>, input: &NewInterview) -> AppResult<Interview> {
        Self::valider(input)?;
        self.repo.save_and_mark_candidate(id, input)
    }

    /// Supprime un entretien.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn delete(&self, id: Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }

    /// Enregistre l'analyse `IA` du compte rendu.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn save_analysis(&self, id: Uuid, analysis: &InterviewAnalysis) -> AppResult<()> {
        self.repo.save_analysis(id, analysis)
    }

    /// Règles de validation d'un entretien.
    ///
    /// La date porte une heure et n'est donc pas au format `AAAA-MM-JJ` des candidatures :
    /// elle est comparée au format `RFC 3339` que produit le formulaire, seul format que les
    /// requêtes de plage du calendrier savent borner correctement.
    fn valider(input: &NewInterview) -> AppResult<()> {
        if input.application_id.is_nil() {
            return Err(AppError::Validation(
                "La candidature concernée est requise".into(),
            ));
        }
        if chrono::DateTime::parse_from_rfc3339(&input.interview_date).is_err() {
            return Err(AppError::Validation(
                "La date et l'heure de l'entretien sont invalides".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
