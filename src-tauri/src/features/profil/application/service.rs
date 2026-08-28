//! Validation et complétion du profil professionnel.

use crate::core::errors::{AppError, AppResult};
use crate::core::utils::validation::validate_optional_http_url;
use crate::features::profil::domain::{Identite, Profil, ProfilCharge, ProfilRepository};

/// Service métier du profil, générique sur son dépôt.
pub struct ProfilService<R: ProfilRepository> {
    repo: R,
}

impl<R: ProfilRepository> ProfilService<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Charge le profil avec les informations de complétion nécessaires à l'écran.
    pub fn charger(&self) -> AppResult<ProfilCharge> {
        let (profil, updated_at) = self.repo.obtenir()?;
        Ok(enrichir(profil, updated_at))
    }

    /// Valide et remplace le profil complet.
    pub fn enregistrer(&self, profil: &Profil) -> AppResult<ProfilCharge> {
        valider(profil)?;
        let (profil, updated_at) = self.repo.enregistrer(profil)?;
        Ok(enrichir(profil, Some(updated_at)))
    }
}

fn enrichir(profil: Profil, updated_at: Option<String>) -> ProfilCharge {
    let sections = sections_completes(&profil);
    let complete = sections.iter().filter(|(_, complete)| *complete).count() as u16;
    let completion = ((complete * 100 + 3) / 7) as u8;
    ProfilCharge {
        profil,
        completion,
        sections_incompletes: sections
            .into_iter()
            .filter(|(_, complete)| !complete)
            .map(|(label, _)| label.to_owned())
            .collect(),
        updated_at,
    }
}

fn sections_completes(profil: &Profil) -> [(&'static str, bool); 7] {
    [
        ("votre identité", identite_complete(&profil.identite)),
        (
            "une expérience",
            profil.experiences.iter().any(|item| {
                !item.intitule.trim().is_empty()
                    && !item.entreprise.trim().is_empty()
                    && !item.date_debut.trim().is_empty()
            }),
        ),
        (
            "vos compétences",
            profil
                .competences
                .iter()
                .any(|item| !item.nom.trim().is_empty()),
        ),
        (
            "une formation",
            profil.formations.iter().any(|item| {
                !item.diplome.trim().is_empty() && !item.etablissement.trim().is_empty()
            }),
        ),
        (
            "une langue",
            profil
                .langues
                .iter()
                .any(|item| !item.nom.trim().is_empty() && !item.niveau.trim().is_empty()),
        ),
        (
            "un projet",
            profil
                .projets
                .iter()
                .any(|item| !item.nom.trim().is_empty()),
        ),
        (
            "une certification",
            profil
                .certifications
                .iter()
                .any(|item| !item.nom.trim().is_empty()),
        ),
    ]
}

fn identite_complete(identite: &Identite) -> bool {
    !identite.prenom.trim().is_empty()
        && !identite.nom.trim().is_empty()
        && !identite.email.trim().is_empty()
}

fn valider(profil: &Profil) -> AppResult<()> {
    let email = profil.identite.email.trim();
    if !email.is_empty() && !email_valide(email) {
        return Err(AppError::Validation("L'adresse e-mail est invalide".into()));
    }
    validate_optional_http_url(profil.identite.linkedin.as_deref(), "Le profil LinkedIn")?;
    validate_optional_http_url(profil.identite.github.as_deref(), "Le profil GitHub")?;
    validate_optional_http_url(profil.identite.site_web.as_deref(), "Le site web")?;

    for experience in &profil.experiences {
        if experience.intitule.trim().is_empty()
            || experience.entreprise.trim().is_empty()
            || experience.date_debut.trim().is_empty()
        {
            return Err(AppError::Validation(
                "Chaque expérience nécessite un intitulé, une entreprise et une date de début"
                    .into(),
            ));
        }
        if experience.poste_actuel && experience.date_fin.is_some() {
            return Err(AppError::Validation(
                "Un poste actuel ne peut pas avoir de date de fin".into(),
            ));
        }
    }
    if profil
        .competences
        .iter()
        .any(|item| item.nom.trim().is_empty())
    {
        return Err(AppError::Validation(
            "Chaque compétence nécessite un nom".into(),
        ));
    }
    if profil
        .formations
        .iter()
        .any(|item| item.diplome.trim().is_empty() || item.etablissement.trim().is_empty())
    {
        return Err(AppError::Validation(
            "Chaque formation nécessite un diplôme et un établissement".into(),
        ));
    }
    if profil
        .langues
        .iter()
        .any(|item| item.nom.trim().is_empty() || item.niveau.trim().is_empty())
    {
        return Err(AppError::Validation(
            "Chaque langue nécessite un nom et un niveau".into(),
        ));
    }
    if profil.projets.iter().any(|item| item.nom.trim().is_empty()) {
        return Err(AppError::Validation(
            "Chaque projet nécessite un nom".into(),
        ));
    }
    if profil
        .certifications
        .iter()
        .any(|item| item.nom.trim().is_empty())
    {
        return Err(AppError::Validation(
            "Chaque certification nécessite un nom".into(),
        ));
    }
    for projet in &profil.projets {
        validate_optional_http_url(projet.url.as_deref(), "Le lien du projet")?;
    }
    for certification in &profil.certifications {
        validate_optional_http_url(certification.url.as_deref(), "Le lien de la certification")?;
    }
    Ok(())
}

fn email_valide(email: &str) -> bool {
    email.split_once('@').is_some_and(|(local, domaine)| {
        !local.is_empty()
            && domaine.contains('.')
            && !domaine.starts_with('.')
            && !domaine.ends_with('.')
    })
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
