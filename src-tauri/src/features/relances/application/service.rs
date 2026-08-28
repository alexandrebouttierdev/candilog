//! Cas d'usage des relances.

use crate::core::errors::{AppError, AppResult};
use crate::features::relances::domain::{NouvelleRelance, Relance, RelanceRepository};
use uuid::Uuid;

/// Service métier des relances, générique sur le dépôt.
pub struct RelanceService<R: RelanceRepository> {
    repo: R,
}

impl<R: RelanceRepository> RelanceService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Liste toutes les relances.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn lister(&self) -> AppResult<Vec<Relance>> {
        self.repo.list()
    }

    /// Liste les relances d'une plage de dates, bornes incluses.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn lister_entre(&self, from: &str, to: &str) -> AppResult<Vec<Relance>> {
        self.repo.list_between(from, to)
    }

    /// Valide puis crée la relance.
    ///
    /// # Errors
    /// `AppError::Validation` si la candidature ou la date manque.
    pub fn creer(&self, input: &NouvelleRelance) -> AppResult<Relance> {
        Self::valider(input)?;
        self.repo.create(input)
    }

    /// Valide puis met à jour la relance.
    ///
    /// # Errors
    /// `AppError::Validation` si un champ requis manque ;
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn modifier(&self, id: Uuid, input: &NouvelleRelance) -> AppResult<Relance> {
        Self::valider(input)?;
        self.repo.update(id, input)
    }

    /// Supprime une relance.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn supprimer(&self, id: Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }

    /// Règles de validation d'une relance.
    ///
    /// La date est au format `AAAA-MM-JJ`, sans heure : une relance se programme au jour,
    /// et c'est ce format que les requêtes de plage du calendrier savent borner.
    fn valider(input: &NouvelleRelance) -> AppResult<()> {
        if input.candidature_id.is_nil() {
            return Err(AppError::Validation(
                "La candidature concernée est requise".into(),
            ));
        }
        if chrono::NaiveDate::parse_from_str(&input.date_relance, "%Y-%m-%d").is_err() {
            return Err(AppError::Validation(
                "La date de relance est invalide".into(),
            ));
        }
        if input.type_relance.trim().is_empty() {
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
