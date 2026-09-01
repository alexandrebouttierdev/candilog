//! Validation et complétion du profil professionnel.

use crate::core::errors::{AppError, AppResult};
use crate::core::utils::text::search_key;
use crate::core::utils::validation::validate_optional_http_url;
use crate::features::profile::domain::{
    apply_decisions, build_preview, Identity, ImportProfilePreview, ImportProfileRequest,
    ImportProfileResult, Profile, ProfilePayload, ProfileRepository, Skill,
};

/// Service métier du profil, générique sur son dépôt.
pub struct ProfileService<R: ProfileRepository> {
    repo: R,
}

impl<R: ProfileRepository> ProfileService<R> {
    #[must_use]
    pub const fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Payload le profil avec les informations de complétion nécessaires à l'écran.
    pub fn load(&self) -> AppResult<ProfilePayload> {
        let (profile, updated_at) = self.repo.get()?;
        Ok(enrichir(profile, updated_at))
    }

    /// Valide et remplace le profil complet.
    pub fn save(&self, profile: &Profile) -> AppResult<ProfilePayload> {
        valider(profile)?;
        let (profile, updated_at) = self.repo.save(profile)?;
        Ok(enrichir(profile, Some(updated_at)))
    }

    /// Compare un CV extrait au profil actuel sans rien écrire.
    pub fn preview_import(&self, extracted: &Profile) -> AppResult<ImportProfilePreview> {
        let (current, _) = self.repo.get()?;
        Ok(build_preview(&current, extracted))
    }

    /// Applique les décisions d'import en une seule écriture.
    pub fn apply_import(&self, request: &ImportProfileRequest) -> AppResult<ImportProfileResult> {
        let (current, _) = self.repo.get()?;
        let (merged, result) = apply_decisions(&current, request)?;
        valider(&merged)?;
        self.repo.save(&merged)?;
        Ok(result)
    }

    /// Ajoute une compétence au profil, sans doublon selon une comparaison normalisée
    /// (casse, accents, espaces) : l'éditeur de CV propose d'ajouter une compétence attendue
    /// par l'offre, et rejouer l'action ne doit jamais créer d'entrée en double.
    ///
    /// # Errors
    /// `AppError::Validation` si le nom, une fois retiré de ses espaces, est vide.
    pub fn add_skill(&self, name: &str) -> AppResult<ProfilePayload> {
        let name = name.trim();
        if name.is_empty() {
            return Err(AppError::Validation("La compétence est requise".into()));
        }
        let (mut profile, _) = self.repo.get()?;
        if !profile
            .skills
            .iter()
            .any(|skill| search_key(&skill.name) == search_key(name))
        {
            profile.skills.push(Skill { name: name.into() });
        }
        self.save(&profile)
    }
}

fn enrichir(profile: Profile, updated_at: Option<String>) -> ProfilePayload {
    let sections = sections_complete(&profile);
    let complete = sections.iter().filter(|(_, complete)| *complete).count() as u16;
    let completion = ((complete * 100 + 3) / 7) as u8;
    ProfilePayload {
        profile,
        completion,
        incomplete_sections: sections
            .into_iter()
            .filter(|(_, complete)| !complete)
            .map(|(label, _)| label.to_owned())
            .collect(),
        updated_at,
    }
}

fn sections_complete(profile: &Profile) -> [(&'static str, bool); 7] {
    [
        ("votre identité", identity_complete(&profile.identity)),
        (
            "une expérience",
            profile.experiences.iter().any(|item| {
                !item.title.trim().is_empty()
                    && !item.company.trim().is_empty()
                    && !item.start_date.trim().is_empty()
            }),
        ),
        (
            "vos compétences",
            profile
                .skills
                .iter()
                .any(|item| !item.name.trim().is_empty()),
        ),
        (
            "une formation",
            profile
                .education
                .iter()
                .any(|item| !item.degree.trim().is_empty() && !item.school.trim().is_empty()),
        ),
        (
            "une langue",
            profile
                .languages
                .iter()
                .any(|item| !item.name.trim().is_empty() && !item.level.trim().is_empty()),
        ),
        (
            "un projet",
            profile
                .projects
                .iter()
                .any(|item| !item.name.trim().is_empty()),
        ),
        (
            "une certification",
            profile
                .certifications
                .iter()
                .any(|item| !item.name.trim().is_empty()),
        ),
    ]
}

fn identity_complete(identity: &Identity) -> bool {
    !identity.first_name.trim().is_empty()
        && !identity.name.trim().is_empty()
        && !identity.email.trim().is_empty()
}

fn valider(profile: &Profile) -> AppResult<()> {
    let email = profile.identity.email.trim();
    if !email.is_empty() && !email_valide(email) {
        return Err(AppError::Validation("L'adresse e-mail est invalide".into()));
    }
    validate_optional_http_url(profile.identity.linkedin.as_deref(), "Le profil LinkedIn")?;
    validate_optional_http_url(profile.identity.github.as_deref(), "Le profil GitHub")?;
    validate_optional_http_url(profile.identity.website.as_deref(), "Le site web")?;

    for experience in &profile.experiences {
        if experience.title.trim().is_empty()
            || experience.company.trim().is_empty()
            || experience.start_date.trim().is_empty()
        {
            return Err(AppError::Validation(
                "Chaque expérience nécessite un intitulé, une entreprise et une date de début"
                    .into(),
            ));
        }
        if experience.current && experience.end_date.is_some() {
            return Err(AppError::Validation(
                "Un poste actuel ne peut pas avoir de date de fin".into(),
            ));
        }
    }
    if profile
        .skills
        .iter()
        .any(|item| item.name.trim().is_empty())
    {
        return Err(AppError::Validation(
            "Chaque compétence nécessite un nom".into(),
        ));
    }
    if profile
        .education
        .iter()
        .any(|item| item.degree.trim().is_empty() || item.school.trim().is_empty())
    {
        return Err(AppError::Validation(
            "Chaque formation nécessite un diplôme et un établissement".into(),
        ));
    }
    if profile
        .languages
        .iter()
        .any(|item| item.name.trim().is_empty() || item.level.trim().is_empty())
    {
        return Err(AppError::Validation(
            "Chaque langue nécessite un nom et un niveau".into(),
        ));
    }
    if profile
        .projects
        .iter()
        .any(|item| item.name.trim().is_empty())
    {
        return Err(AppError::Validation(
            "Chaque projet nécessite un nom".into(),
        ));
    }
    if profile
        .certifications
        .iter()
        .any(|item| item.name.trim().is_empty())
    {
        return Err(AppError::Validation(
            "Chaque certification nécessite un nom".into(),
        ));
    }
    for project in &profile.projects {
        validate_optional_http_url(project.url.as_deref(), "Le lien du projet")?;
    }
    for certification in &profile.certifications {
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
