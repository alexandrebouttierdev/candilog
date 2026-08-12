//! Types du domaine du profil utilisateur (partagés entre les modules `profil` et `ia`).

use serde::{Deserialize, Serialize};

/// Informations personnelles du candidat.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PersonalInfo {
    /// Prénom.
    pub first_name: String,
    /// Nom.
    pub last_name: String,
    /// Adresse email.
    pub email: String,
    /// Téléphone.
    pub phone: Option<String>,
    /// Ville.
    pub city: Option<String>,
    /// Titre / accroche professionnelle.
    pub headline: Option<String>,
    /// Résumé / présentation.
    pub summary: Option<String>,
    /// Profil `LinkedIn`.
    pub linkedin: Option<String>,
    /// Profil `GitHub`.
    pub github: Option<String>,
    /// Site web / portfolio.
    pub website: Option<String>,
}

/// Expérience professionnelle.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Experience {
    /// Intitulé du poste.
    pub title: String,
    /// Entreprise.
    pub company: String,
    /// Lieu.
    pub location: Option<String>,
    /// Date de début (texte libre, ex. « 2023-06 »).
    pub start_date: String,
    /// Date de fin (texte libre) ; absente si poste en cours.
    pub end_date: Option<String>,
    /// Poste occupé actuellement.
    pub current: bool,
    /// Description / réalisations.
    pub description: Option<String>,
}

impl Experience {
    #[must_use]
    pub fn is_complete(&self) -> bool {
        !self.title.trim().is_empty() && !self.company.trim().is_empty()
    }
}

/// Compétence.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Skill {
    /// Nom de la compétence.
    pub name: String,
}

impl Skill {
    #[must_use]
    pub fn is_complete(&self) -> bool {
        !self.name.trim().is_empty()
    }
}

/// Formation.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Education {
    /// Diplôme.
    pub degree: String,
    /// Établissement.
    pub school: String,
    /// Lieu.
    pub location: Option<String>,
    /// Date de début (texte libre).
    pub start_date: Option<String>,
    /// Date de fin (texte libre).
    pub end_date: Option<String>,
    /// Description.
    pub description: Option<String>,
}

impl Education {
    #[must_use]
    pub fn is_complete(&self) -> bool {
        !self.degree.trim().is_empty() && !self.school.trim().is_empty()
    }
}

/// Langue parlée.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Language {
    /// Nom de la langue.
    pub name: String,
    /// Niveau (texte libre, ex. « C1 », « natif »).
    pub level: String,
}

impl Language {
    #[must_use]
    pub fn is_complete(&self) -> bool {
        !self.name.trim().is_empty() && !self.level.trim().is_empty()
    }
}

/// Projet personnel ou professionnel.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Project {
    /// Nom du projet.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Lien.
    pub url: Option<String>,
    /// Technologies (texte libre, ex. « React, Rust »).
    pub technologies: Option<String>,
}

impl Project {
    #[must_use]
    pub fn is_complete(&self) -> bool {
        !self.name.trim().is_empty()
    }
}

/// Certification.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Certification {
    /// Nom de la certification.
    pub name: String,
    /// Organisme émetteur.
    pub issuer: Option<String>,
    /// Date d'obtention (texte libre).
    pub date: Option<String>,
    /// Lien.
    pub url: Option<String>,
}

impl Certification {
    #[must_use]
    pub fn is_complete(&self) -> bool {
        !self.name.trim().is_empty()
    }
}

/// Profil complet de l'utilisateur (persisté en `JSON` dans `profiles.data`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Profile {
    /// Informations personnelles.
    pub personal: PersonalInfo,
    /// Expériences professionnelles.
    pub experiences: Vec<Experience>,
    /// Compétences.
    pub skills: Vec<Skill>,
    /// Formations.
    pub education: Vec<Education>,
    /// Langues.
    pub languages: Vec<Language>,
    /// Projets.
    pub projects: Vec<Project>,
    /// Certifications.
    pub certifications: Vec<Certification>,
}

#[cfg(test)]
#[path = "tests/profile/mod.rs"]
mod tests;
