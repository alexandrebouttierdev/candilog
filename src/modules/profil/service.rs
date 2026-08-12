//! Logique métier du profil.

use crate::modules::profil::repository::ProfilRepository;
use crate::shared::error::{AppError, AppResult};
use crate::shared::profile::Profile;

/// Service métier du profil, générique sur le dépôt (testable via mock).
pub struct ProfilService<R: ProfilRepository> {
    repo: R,
}

impl<R: ProfilRepository> ProfilService<R> {
    /// Construit le service avec son dépôt.
    #[must_use]
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// Récupère le profil (profil par défaut si aucun n'a encore été enregistré).
    ///
    /// # Errors
    /// `AppError::Serialization` si le contenu stocké est invalide ; sinon l'erreur du dépôt.
    pub fn get(&self) -> AppResult<Profile> {
        self.repo.get()
    }

    /// Valide puis persiste le profil.
    ///
    /// # Errors
    /// `AppError::Validation` si le profil est invalide ; `AppError::Serialization` si le profil
    /// ne peut pas être sérialisé ; sinon l'erreur du dépôt.
    pub fn update(&self, profil: &Profile) -> AppResult<Profile> {
        validate(profil)?;
        self.repo.upsert(profil)
    }
}

/// Valide un profil (email si fourni, expériences complètes).
fn validate(profil: &Profile) -> AppResult<()> {
    let email = profil.personal.email.trim();
    if !email.is_empty() && !is_valid_email(email) {
        return Err(AppError::Validation("L'adresse email est invalide".into()));
    }
    for exp in &profil.experiences {
        if !exp.is_complete() {
            return Err(AppError::Validation(
                "Chaque expérience nécessite un intitulé et une entreprise".into(),
            ));
        }
    }
    if profil.skills.iter().any(|item| !item.is_complete()) {
        return Err(AppError::Validation(
            "Chaque compétence nécessite un nom".into(),
        ));
    }
    if profil.education.iter().any(|item| !item.is_complete()) {
        return Err(AppError::Validation(
            "Chaque formation nécessite un diplôme et un établissement".into(),
        ));
    }
    if profil.languages.iter().any(|item| !item.is_complete()) {
        return Err(AppError::Validation(
            "Chaque langue nécessite un nom et un niveau".into(),
        ));
    }
    if profil.projects.iter().any(|item| !item.is_complete()) {
        return Err(AppError::Validation(
            "Chaque projet nécessite un nom".into(),
        ));
    }
    if profil.certifications.iter().any(|item| !item.is_complete()) {
        return Err(AppError::Validation(
            "Chaque certification nécessite un nom".into(),
        ));
    }
    Ok(())
}

/// Vérifie sommairement le format d'un email (présence d'un `@` avec un `.` après).
fn is_valid_email(email: &str) -> bool {
    match email.split_once('@') {
        Some((local, domain)) => {
            !local.is_empty()
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
        }
        None => false,
    }
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
