//! Cas d'usage des candidatures.

use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::{Page, MAX_PAGE_SIZE};
use crate::core::utils::validation::validate_optional_http_url;
use crate::features::applications::domain::{
    Application, ApplicationFilter, ApplicationRepository, ApplicationStatus, ApplicationType,
    NewApplication, PipelineBreakdown, MAX_WEEKLY_HOURS,
};
use uuid::Uuid;

/// Service métier des candidatures, générique sur le dépôt.
pub struct ApplicationService<R: ApplicationRepository> {
    repo: R,
}

impl<R: ApplicationRepository> ApplicationService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Dépôt sous-jacent, pour observer en test ce que le service lui transmet.
    ///
    /// La normalisation (lien de l'offre effacé pour une spontanée) ne se voit pas dans la
    /// valeur de retour, qui vient du dépôt : seule la saisie reçue en témoigne.
    #[cfg(test)]
    pub(crate) const fn repository(&self) -> &R {
        &self.repo
    }

    /// Liste toutes les candidatures.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list(&self) -> AppResult<Vec<Application>> {
        self.repo.list()
    }

    /// Récupère une candidature par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn get(&self, id: Uuid) -> AppResult<Application> {
        self.repo.get(id)
    }

    /// Renvoie une page filtrée et triée.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list_page(
        &self,
        page: u64,
        page_size: u64,
        filter: &ApplicationFilter,
    ) -> AppResult<Page<Application>> {
        self.repo.list_page(page, page_size, filter)
    }

    /// Toutes les candidatures du filtre, page par page, pour un export complet.
    ///
    /// `list_page` plafonne à `MAX_PAGE_SIZE` : un export qui s'arrêterait à la première
    /// page tronquerait silencieusement le CSV tout en renvoyant `total` comme si tout
    /// avait été écrit.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn list_matching(&self, filter: &ApplicationFilter) -> AppResult<Vec<Application>> {
        let mut page = 1;
        let mut items = Vec::new();
        loop {
            let chunk = self.repo.list_page(page, MAX_PAGE_SIZE, filter)?;
            let received = chunk.items.len();
            let total = chunk.total;
            items.extend(chunk.items);
            if items.len() as u64 >= total || received == 0 {
                break;
            }
            page += 1;
        }
        Ok(items)
    }

    /// Compte les candidatures par statut, pour les en-têtes de colonnes du Kanban.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn breakdown(&self, filter: &ApplicationFilter) -> AppResult<PipelineBreakdown> {
        self.repo.breakdown(filter)
    }

    /// Valide, normalise puis crée la candidature.
    ///
    /// # Errors
    /// `AppError::Validation` si le poste est vide, la date invalide, le contrat absent,
    /// le volume horaire aberrant ou le lien de l'offre mal formé.
    pub fn create(&self, input: &NewApplication) -> AppResult<Application> {
        let input = Self::normalize(input)?;
        self.repo.create(&input)
    }

    /// Valide, normalise puis met à jour la candidature.
    ///
    /// # Errors
    /// `AppError::Validation` si un champ est invalide ; `AppError::NotFound` si
    /// l'identifiant est inconnu.
    pub fn update(&self, id: Uuid, input: &NewApplication) -> AppResult<Application> {
        let input = Self::normalize(input)?;
        self.repo.update(id, &input)
    }

    /// Change le statut d'une candidature — le geste du glisser-déposer du Kanban.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn change_status(&self, id: Uuid, status: ApplicationStatus) -> AppResult<Application> {
        self.repo.update_status(id, status)
    }

    /// Supprime une candidature.
    ///
    /// # Errors
    /// Propage l'erreur du dépôt.
    pub fn delete(&self, id: Uuid) -> AppResult<()> {
        self.repo.delete(id)
    }

    /// Valide les règles communes à la création et à la modification, puis renvoie la
    /// candidature sous sa forme canonique.
    ///
    /// Validation **et** normalisation au même endroit : la seconde dépend directement de
    /// la première (le lien de l'offre n'est vérifié que là où il est permis), et les
    /// séparer laisserait exister un état intermédiaire validé mais non canonique.
    ///
    /// La date est comparée au format `AAAA-MM-JJ` attendu par les requêtes de plage : une
    /// date stockée dans un autre format ferait échouer silencieusement les filtres de
    /// période, qui comparent des chaînes.
    fn normalize(input: &NewApplication) -> AppResult<NewApplication> {
        if input.job_title.trim().is_empty() {
            return Err(AppError::Validation("Le poste est requis".into()));
        }
        if input.company_id.is_nil() {
            return Err(AppError::Validation("L'entreprise est requise".into()));
        }
        if input.contract_type_code.trim().is_empty() {
            return Err(AppError::Validation("Le type de contrat est requis".into()));
        }
        if chrono::NaiveDate::parse_from_str(&input.sent_date, "%Y-%m-%d").is_err() {
            return Err(AppError::Validation("La date d'envoi est invalide".into()));
        }
        Self::valider_heures(input.weekly_hours)?;

        let mut normalisee = input.clone();
        match input.application_type {
            // Le lien est la trace de l'offre à laquelle on a répondu : sans lui, la
            // candidature n'est plus rattachable à une annonce, et la relire six mois plus
            // tard ne dit plus à quoi elle correspondait.
            ApplicationType::JobOffer => {
                let job_url = input
                    .job_url
                    .as_deref()
                    .map(str::trim)
                    .filter(|url| !url.is_empty());
                if job_url.is_none() {
                    return Err(AppError::Validation(
                        "Le lien de l'offre est requis pour une candidature à une offre".into(),
                    ));
                }
                validate_optional_http_url(job_url, "Le lien de l'offre")?;
                normalisee.job_url = job_url.map(str::to_owned);
            }
            // Une candidature spontanée n'a pas d'offre : conserver le lien d'un ancien
            // état « offre » ferait pointer la fiche vers une annonce sans rapport.
            ApplicationType::Unsolicited => normalisee.job_url = None,
        }
        Ok(normalisee)
    }

    /// Contrôle le volume horaire hebdomadaire lorsqu'il est renseigné.
    ///
    /// `NaN` et l'infini sont refusés explicitement : `f64` les accepte, `JSON` les laisse
    /// passer via une chaîne, et une comparaison avec `NaN` est toujours fausse — la borne
    /// haute seule les laisserait donc entrer en base.
    fn valider_heures(weekly_hours: Option<f64>) -> AppResult<()> {
        let Some(hours) = weekly_hours else {
            return Ok(());
        };
        if !hours.is_finite() {
            return Err(AppError::Validation(
                "Le nombre d'heures par semaine est invalide".into(),
            ));
        }
        if hours <= 0.0 || hours > MAX_WEEKLY_HOURS {
            return Err(AppError::Validation(format!(
                "Le nombre d'heures par semaine doit être compris entre 0 et {MAX_WEEKLY_HOURS:.0}"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
