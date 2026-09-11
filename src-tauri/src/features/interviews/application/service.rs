//! Cas d'usage des entretiens.

use crate::core::errors::{AppError, AppResult};
use crate::features::interviews::domain::{
    Interview, InterviewAnalysis, InterviewRepository, NewInterview,
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

    /// Liste tous les entretiens.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list(&self) -> AppResult<Vec<Interview>> {
        self.repo.list()
    }

    /// Liste les entretiens d'une plage de dates, bornes incluses.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list_between(&self, from: &str, to: &str) -> AppResult<Vec<Interview>> {
        Self::validate_range(from, to)?;
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

    /// Valide une plage calendaire reçue de l'IPC (`from ≤ to`).
    ///
    /// Le calendrier envoie `AAAA-MM-JJTHH:MM:SS` sans fuseau ; le formulaire d'entretien
    /// envoie du RFC 3339. Les deux formes sont acceptées.
    fn validate_range(from: &str, to: &str) -> AppResult<()> {
        let from_dt = Self::parse_bound(from, "début")?;
        let to_dt = Self::parse_bound(to, "fin")?;
        if from_dt > to_dt {
            return Err(AppError::Validation(
                "La date de début doit précéder la date de fin".into(),
            ));
        }
        Ok(())
    }

    fn parse_bound(value: &str, label: &str) -> AppResult<chrono::DateTime<chrono::FixedOffset>> {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(value) {
            return Ok(dt);
        }
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S") {
            return Ok(naive.and_utc().fixed_offset());
        }
        Err(AppError::Validation(format!(
            "La date de {label} de la plage est invalide"
        )))
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
