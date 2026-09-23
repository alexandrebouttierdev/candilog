//! Cas d'usage des candidatures.

use crate::core::errors::{AppError, AppResult};
use crate::core::pagination::{Page, MAX_PAGE_SIZE};
use crate::core::utils::validation::validate_optional_http_url;
use crate::features::applications::domain::{
    Application, ApplicationChannel, ApplicationFilter, ApplicationRepository, ApplicationStatus,
    DeletionImpact, NewApplication, PipelineBreakdown, StatusChange, MAX_WEEKLY_HOURS,
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
        Self::validate_filter(filter)?;
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
        Self::validate_filter(filter)?;
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
        Self::validate_filter(filter)?;
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

    /// Ce que la suppression emporterait, pour l'énumérer avant de confirmer.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn deletion_impact(&self, id: Uuid) -> AppResult<DeletionImpact> {
        self.repo.deletion_impact(id)
    }

    /// Historique des statuts, le plus récent d'abord.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    pub fn status_history(&self, id: Uuid) -> AppResult<Vec<StatusChange>> {
        self.repo.status_history(id)
    }

    /// Duplique une candidature pour en préparer une voisine (`⌘D`).
    ///
    /// La copie reprend le poste, l'entreprise, le contact, le canal, le contrat et les
    /// précisions ; elle repart **En attente**, envoyée aujourd'hui, avec une nouvelle
    /// référence. Relances, entretiens et historique ne sont pas copiés : ils décrivent ce
    /// qui est arrivé à l'original, pas à la copie.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu ; les erreurs de validation de
    /// [`Self::create`].
    pub fn duplicate(&self, id: Uuid) -> AppResult<Application> {
        let source = self.repo.get(id)?;
        let copie = NewApplication {
            job_title: source.job_title,
            company_id: source.company_id,
            contact_id: source.contact_id,
            channel: source.channel,
            contract_type_code: source.contract_type_code,
            weekly_work_schedule: source.weekly_work_schedule,
            weekly_hours: source.weekly_hours,
            professional_domain_id: source.professional_domain_id,
            city: source.city,
            address: source.address,
            company_type_id: source.company_type_id,
            status: ApplicationStatus::Pending,
            sent_date: chrono::Local::now()
                .date_naive()
                .format("%Y-%m-%d")
                .to_string(),
            job_url: source.job_url,
            notes: source.notes,
        };
        self.create(&copie)
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
        let job_url = input
            .job_url
            .as_deref()
            .map(str::trim)
            .filter(|url| !url.is_empty());
        match input.channel {
            // Le lien est la trace de l'offre publiée à laquelle on a répondu : sans lui, la
            // candidature n'est plus rattachable à une annonce, et la relire six mois plus
            // tard ne dit plus à quoi elle correspondait.
            ApplicationChannel::Offer => {
                if job_url.is_none() {
                    return Err(AppError::Validation(
                        "Le lien de l'offre est requis pour une candidature à une offre".into(),
                    ));
                }
                validate_optional_http_url(job_url, "Le lien de l'offre")?;
                normalisee.job_url = job_url.map(str::to_owned);
            }
            // Site de l'entreprise ou réseau : l'offre n'a pas toujours d'adresse publique
            // (cooptation, annonce retirée). Le lien reste facultatif, mais valide s'il est
            // donné.
            ApplicationChannel::CompanySite | ApplicationChannel::Network => {
                validate_optional_http_url(job_url, "Le lien de l'offre")?;
                normalisee.job_url = job_url.map(str::to_owned);
            }
            // Une candidature spontanée n'a pas d'offre : conserver le lien d'un ancien
            // état « offre » ferait pointer la fiche vers une annonce sans rapport.
            ApplicationChannel::Spontaneous => normalisee.job_url = None,
        }
        Ok(normalisee)
    }

    /// Revalide les bornes de filtre reçues de l'IPC avant toute requête SQL.
    fn validate_filter(filter: &ApplicationFilter) -> AppResult<()> {
        for (value, label) in [
            (&filter.start_date, "La date de début"),
            (&filter.end_date, "La date de fin"),
        ] {
            if let Some(raw) = value {
                if chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d").is_err() {
                    return Err(AppError::Validation(format!(
                        "{label} du filtre est invalide"
                    )));
                }
            }
        }
        if let (Some(start), Some(end)) = (&filter.start_date, &filter.end_date) {
            if start > end {
                return Err(AppError::Validation(
                    "La date de début doit précéder la date de fin".into(),
                ));
            }
        }
        Self::validate_filter_hours(filter.min_weekly_hours, "minimal")?;
        Self::validate_filter_hours(filter.max_weekly_hours, "maximal")?;
        if let (Some(min), Some(max)) = (filter.min_weekly_hours, filter.max_weekly_hours) {
            if min > max {
                return Err(AppError::Validation(
                    "Le volume horaire minimal ne peut pas dépasser le maximal".into(),
                ));
            }
        }
        Ok(())
    }

    fn validate_filter_hours(value: Option<f64>, label: &str) -> AppResult<()> {
        let Some(hours) = value else {
            return Ok(());
        };
        if !hours.is_finite() {
            return Err(AppError::Validation(format!(
                "Le volume horaire {label} du filtre est invalide"
            )));
        }
        if !(0.0..=MAX_WEEKLY_HOURS).contains(&hours) {
            return Err(AppError::Validation(format!(
                "Le volume horaire {label} doit être compris entre 0 et {MAX_WEEKLY_HOURS:.0}"
            )));
        }
        Ok(())
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
