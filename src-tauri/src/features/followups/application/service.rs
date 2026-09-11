//! Cas d'usage des relances.

use crate::core::errors::{AppError, AppResult};
use crate::features::followups::domain::{FollowUp, FollowUpRepository, NewFollowUp};
use uuid::Uuid;

/// Service métier des relances, générique sur le dépôt.
pub struct FollowUpService<R: FollowUpRepository> {
    repo: R,
}

impl<R: FollowUpRepository> FollowUpService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Liste toutes les relances.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list(&self) -> AppResult<Vec<FollowUp>> {
        self.repo.list()
    }

    /// Liste les relances d'une plage de dates, bornes incluses.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list_between(&self, from: &str, to: &str) -> AppResult<Vec<FollowUp>> {
        Self::validate_range(from, to)?;
        self.repo.list_between(from, to)
    }

    /// Valide puis crée la relance.
    ///
    /// # Errors
    /// `AppError::Validation` si la candidature ou la date manque.
    pub fn create(&self, input: &NewFollowUp) -> AppResult<FollowUp> {
        Self::valider(input)?;
        self.repo.create(input)
    }

    /// Valide puis met à jour la relance.
    ///
    /// # Errors
    /// `AppError::Validation` si un champ requis manque ;
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn update(&self, id: Uuid, input: &NewFollowUp) -> AppResult<FollowUp> {
        Self::valider(input)?;
        self.repo.update(id, input)
    }

    /// Supprime une relance.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn delete(&self, id: Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }

    /// Valide une plage calendaire reçue de l'IPC (`AAAA-MM-JJ`, `from ≤ to`).
    fn validate_range(from: &str, to: &str) -> AppResult<()> {
        let from_d = chrono::NaiveDate::parse_from_str(from, "%Y-%m-%d").map_err(|_| {
            AppError::Validation("La date de début de la plage est invalide".into())
        })?;
        let to_d = chrono::NaiveDate::parse_from_str(to, "%Y-%m-%d")
            .map_err(|_| AppError::Validation("La date de fin de la plage est invalide".into()))?;
        if from_d > to_d {
            return Err(AppError::Validation(
                "La date de début doit précéder la date de fin".into(),
            ));
        }
        Ok(())
    }

    /// Règles de validation d'une relance.
    ///
    /// La date est au format `AAAA-MM-JJ`, sans heure : une relance se programme au jour,
    /// et c'est ce format que les requêtes de plage du calendrier savent borner.
    fn valider(input: &NewFollowUp) -> AppResult<()> {
        if input.application_id.is_nil() {
            return Err(AppError::Validation(
                "La candidature concernée est requise".into(),
            ));
        }
        if chrono::NaiveDate::parse_from_str(&input.follow_up_date, "%Y-%m-%d").is_err() {
            return Err(AppError::Validation(
                "La date de relance est invalide".into(),
            ));
        }
        if input.channel.trim().is_empty() {
            return Err(AppError::Validation(
                "Le canal de relance est requis".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
