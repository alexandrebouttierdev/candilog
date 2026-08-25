//! Cas d'usage des candidatures.

use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::Page;
use crate::core::utils::validation::validate_optional_http_url;
use crate::features::candidatures::domain::{
    Candidature, CandidatureRepository, FiltreCandidatures, NouvelleCandidature,
    RepartitionPipeline, StatutCandidature,
};
use uuid::Uuid;

/// Service métier des candidatures, générique sur le dépôt.
pub struct CandidatureService<R: CandidatureRepository> {
    repo: R,
}

impl<R: CandidatureRepository> CandidatureService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Liste toutes les candidatures.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn lister(&self) -> AppResult<Vec<Candidature>> {
        self.repo.list()
    }

    /// Récupère une candidature par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn obtenir(&self, id: Uuid) -> AppResult<Candidature> {
        self.repo.get(id)
    }

    /// Charge une page filtrée et triée.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn lister_page(
        &self,
        page: u64,
        page_size: u64,
        filtre: &FiltreCandidatures,
    ) -> AppResult<Page<Candidature>> {
        self.repo.list_page(page, page_size, filtre)
    }

    /// Compte les candidatures par statut, pour les en-têtes de colonnes du Kanban.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn repartition(&self, filtre: &FiltreCandidatures) -> AppResult<RepartitionPipeline> {
        self.repo.repartition(filtre)
    }

    /// Valide puis crée la candidature.
    ///
    /// # Errors
    /// `AppError::Validation` si le poste est vide, la date invalide ou le lien mal formé.
    pub fn creer(&self, input: &NouvelleCandidature) -> AppResult<Candidature> {
        Self::valider(input)?;
        self.repo.create(input)
    }

    /// Valide puis met à jour la candidature.
    ///
    /// # Errors
    /// `AppError::Validation` si un champ est invalide ; `AppError::NotFound` si
    /// l'identifiant est inconnu.
    pub fn modifier(&self, id: Uuid, input: &NouvelleCandidature) -> AppResult<Candidature> {
        Self::valider(input)?;
        self.repo.update(id, input)
    }

    /// Change le statut d'une candidature — le geste du glisser-déposer du Kanban.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn changer_statut(&self, id: Uuid, statut: StatutCandidature) -> AppResult<Candidature> {
        self.repo.update_statut(id, statut)
    }

    /// Supprime une candidature.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn supprimer(&self, id: Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }

    /// Règles communes à la création et à la modification.
    ///
    /// La date est comparée au format `AAAA-MM-JJ` attendu par les requêtes de plage : une
    /// date stockée dans un autre format ferait échouer silencieusement les filtres de
    /// période, qui comparent des chaînes.
    fn valider(input: &NouvelleCandidature) -> AppResult<()> {
        if input.poste.trim().is_empty() {
            return Err(AppError::Validation("Le poste est requis".into()));
        }
        if chrono::NaiveDate::parse_from_str(&input.date_envoi, "%Y-%m-%d").is_err() {
            return Err(AppError::Validation("La date d'envoi est invalide".into()));
        }
        validate_optional_http_url(input.lien_offre.as_deref(), "Le lien de l'offre")?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
