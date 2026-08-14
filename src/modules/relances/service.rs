//! Logique métier des relances (validation + délégation au dépôt).

use crate::modules::relances::model::{NouvelleRelance, Relance};
use crate::modules::relances::repository::RelanceRepository;
use crate::shared::error::{AppError, AppResult};

/// Service métier des relances, générique sur le dépôt (testable via mock).
pub struct RelanceService<R: RelanceRepository> {
    repo: R,
}

impl<R: RelanceRepository> RelanceService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Liste les relances (triées par date croissante).
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn lister(&self) -> AppResult<Vec<Relance>> {
        self.repo.list()
    }

    /// Charge une fenêtre temporelle bornée pour le calendrier.
    pub fn lister_entre(&self, from: &str, to: &str) -> AppResult<Vec<Relance>> {
        self.repo.list_between(from, to)
    }

    /// Valide (candidature + date requises) puis crée la relance.
    ///
    /// # Errors
    /// `AppError::Validation` si la candidature est nulle, la date vide, ou si la candidature
    /// liée est introuvable ; sinon l'erreur du dépôt.
    pub fn creer(&self, input: &NouvelleRelance) -> AppResult<Relance> {
        Self::valider(input)?;
        self.repo.create(input)
    }

    /// Valide puis met à jour la relance.
    ///
    /// # Errors
    /// `AppError::Validation` si la candidature est nulle, la date vide, ou si la candidature
    /// liée est introuvable ; `AppError::NotFound` si l'identifiant est inconnu ; sinon l'erreur
    /// du dépôt.
    pub fn modifier(&self, id: uuid::Uuid, input: &NouvelleRelance) -> AppResult<Relance> {
        Self::valider(input)?;
        self.repo.update(id, input)
    }

    /// Supprime une relance.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn supprimer(&self, id: uuid::Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }

    /// Valide qu'une relance cible une candidature et porte une date.
    fn valider(input: &NouvelleRelance) -> AppResult<()> {
        if input.candidature_id.is_nil() || input.date_relance.trim().is_empty() {
            return Err(AppError::Validation(
                "La candidature et la date de relance sont requises".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
