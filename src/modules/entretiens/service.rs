//! Logique métier des entretiens (validation + délégation au dépôt).

use crate::modules::entretiens::model::{Entretien, NouvelEntretien};
use crate::modules::entretiens::repository::EntretienRepository;
use crate::shared::error::{AppError, AppResult};
use crate::shared::types::AnalyseEntretien;

/// Service métier des entretiens, générique sur le dépôt (testable via mock).
pub struct EntretienService<R: EntretienRepository> {
    repo: R,
}

impl<R: EntretienRepository> EntretienService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Liste les entretiens (triés par date croissante).
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn lister(&self) -> AppResult<Vec<Entretien>> {
        self.repo.list()
    }

    /// Récupère un entretien par son identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'entretien n'existe pas ; sinon l'erreur du dépôt.
    pub fn obtenir(&self, id: uuid::Uuid) -> AppResult<Entretien> {
        self.repo.get(id)
    }

    /// Persiste l'analyse `IA` du compte rendu sur un entretien.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu ; sinon l'erreur du dépôt.
    pub fn enregistrer_analyse(&self, id: uuid::Uuid, analyse: &AnalyseEntretien) -> AppResult<()> {
        self.repo.enregistrer_analyse(id, analyse)
    }

    /// Valide (candidature + date requises) puis crée l'entretien.
    ///
    /// # Errors
    /// `AppError::Validation` si la candidature est nulle, la date vide, ou si la candidature
    /// ou le contact lié est introuvable ; sinon l'erreur du dépôt.
    pub fn creer(&self, input: &NouvelEntretien) -> AppResult<Entretien> {
        Self::valider(input)?;
        self.repo.create(input)
    }

    /// Valide puis met à jour l'entretien.
    ///
    /// # Errors
    /// `AppError::Validation` si la candidature est nulle, la date vide, ou si la candidature ou
    /// le contact lié est introuvable ; `AppError::NotFound` si l'identifiant est inconnu.
    pub fn modifier(&self, id: uuid::Uuid, input: &NouvelEntretien) -> AppResult<Entretien> {
        Self::valider(input)?;
        self.repo.update(id, input)
    }

    /// Supprime un entretien.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn supprimer(&self, id: uuid::Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }

    /// Valide qu'un entretien cible une candidature et porte une date.
    fn valider(input: &NouvelEntretien) -> AppResult<()> {
        if input.candidature_id.is_nil() || input.date_entretien.trim().is_empty() {
            return Err(AppError::Validation(
                "La candidature et la date de l'entretien sont requises".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
