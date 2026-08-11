//! Logique métier des candidatures.

use crate::modules::candidatures::model::{Candidature, NouvelleCandidature, StatutCandidature};
use crate::modules::candidatures::repository::{
    CandidaturePageQuery, CandidatureRepository, CandidatureStats,
};
use crate::modules::metriques::model::Page;
use crate::shared::error::{AppError, AppResult};
use crate::shared::validation::validate_optional_http_url;
use uuid::Uuid;

/// Service métier des candidatures, générique sur le dépôt (testable via mock).
pub struct CandidatureService<R: CandidatureRepository> {
    repo: R,
}

impl<R: CandidatureRepository> CandidatureService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Valide les champs puis crée la candidature.
    ///
    /// # Errors
    /// `AppError::Validation` si le poste ou la date est invalide, ou si l'entreprise liée est
    /// introuvable ; sinon l'erreur du dépôt.
    pub fn creer(&self, input: &NouvelleCandidature) -> AppResult<Candidature> {
        Self::valider(input)?;
        self.repo.create(input)
    }

    /// Liste les candidatures (les plus récentes d'abord).
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn lister(&self) -> AppResult<Vec<Candidature>> {
        self.repo.list()
    }

    /// Charge une page filtrée sans matérialiser tout le pipeline.
    pub fn lister_page(
        &self,
        page: u64,
        page_size: u64,
        query: &CandidaturePageQuery,
    ) -> AppResult<Page<Candidature>> {
        self.repo.list_page(page, page_size, query)
    }

    /// Renvoie les agrégats globaux sans charger toutes les lignes.
    pub fn statistiques(&self) -> AppResult<CandidatureStats> {
        self.repo.stats()
    }

    /// Valide les champs puis met à jour la candidature.
    ///
    /// # Errors
    /// `AppError::Validation` si le poste, la date ou l'entreprise liée est invalide ;
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn modifier(&self, id: Uuid, input: &NouvelleCandidature) -> AppResult<Candidature> {
        Self::valider(input)?;
        self.repo.update(id, input)
    }

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

    /// Change le statut d'une candidature.
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
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
