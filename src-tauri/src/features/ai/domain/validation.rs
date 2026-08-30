//! Bornes des données envoyées aux fournisseurs IA et reçues de ceux-ci.

use crate::core::errors::{AppError, AppResult};
use crate::features::ai::domain::{
    AtsAnalysis, CoverLetterRequest, GeneratedResume, StructuredListing,
};
use crate::features::profile::domain::Profile;
use serde::Serialize;

pub const MAX_SOURCE_CHARS: usize = 50_000;
pub const MAX_CONTEXT_CHARS: usize = 10_000;
pub const MAX_PROFILE_CHARS: usize = 200_000;
pub const MAX_STRUCTURED_CHARS: usize = 250_000;
pub const MAX_ITEMS: usize = 100;
pub const MAX_ITEM_CHARS: usize = 4_000;

/// Validation obligatoire de chaque objet structuré produit par un fournisseur.
pub trait ValidateAiOutput: Serialize {
    /// # Errors
    /// Refuse les réponses excessives ou incohérentes avant leur utilisation métier.
    fn validate_ai_output(&self) -> AppResult<()>;
}

fn output_error(label: &str) -> AppError {
    AppError::Provider(format!(
        "La réponse IA dépasse la limite autorisée pour {label}."
    ))
}

fn ensure_output_string(value: &str, label: &str) -> AppResult<()> {
    if value.chars().count() > MAX_ITEM_CHARS {
        Err(output_error(label))
    } else {
        Ok(())
    }
}

fn ensure_output_list<T>(values: &[T], label: &str) -> AppResult<()> {
    if values.len() > MAX_ITEMS {
        Err(output_error(label))
    } else {
        Ok(())
    }
}

fn validate_strings<'a>(values: impl IntoIterator<Item = &'a str>, label: &str) -> AppResult<()> {
    for value in values {
        ensure_output_string(value, label)?;
    }
    Ok(())
}

/// Refuse un texte source vide ou supérieur à la borne commune.
///
/// # Errors
/// Retourne une validation destinée à l'utilisateur.
pub fn validate_source_text(value: &str, label: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        Err(AppError::Validation(format!(
            "{label} ne peut pas être vide"
        )))
    } else if value.chars().count() > MAX_SOURCE_CHARS {
        Err(AppError::Validation(format!(
            "{label} dépasse la taille maximale autorisée"
        )))
    } else {
        Ok(())
    }
}

/// Valide les champs libres d'un brief de lettre avant tout appel fournisseur.
///
/// # Errors
/// Retourne une validation si un champ dépasse sa limite.
pub fn validate_cover_letter_request(request: &CoverLetterRequest) -> AppResult<()> {
    for (label, value) in [
        ("le contexte", request.context.as_deref()),
        (
            "la lettre précédente",
            request.previous_cover_letter.as_deref(),
        ),
        ("l'instruction", request.instruction.as_deref()),
    ] {
        if value.is_some_and(|text| text.chars().count() > MAX_CONTEXT_CHARS) {
            return Err(AppError::Validation(format!(
                "La taille maximale autorisée pour {label} est dépassée."
            )));
        }
    }
    for (label, value) in [
        ("l'entreprise", request.company.as_deref()),
        ("le poste", request.job_title.as_deref()),
        ("le ton", request.tone.as_deref()),
        ("la longueur", request.length.as_deref()),
    ] {
        if value.is_some_and(|text| text.chars().count() > MAX_ITEM_CHARS) {
            return Err(AppError::Validation(format!(
                "La taille maximale autorisée pour {label} est dépassée."
            )));
        }
    }
    Ok(())
}

/// Valide la taille sérialisée d'un profil avant de l'envoyer à un fournisseur.
///
/// # Errors
/// Retourne une validation si le profil ou une de ses collections dépasse la borne.
pub fn validate_profile_input(profile: &Profile) -> AppResult<()> {
    let serialized = serde_json::to_string(profile)
        .map_err(|error| AppError::Serialization(error.to_string()))?;
    if serialized.chars().count() > MAX_PROFILE_CHARS {
        return Err(AppError::Validation(
            "Le profil est trop volumineux pour être envoyé à l'assistant IA.".into(),
        ));
    }
    for (label, len) in [
        ("expériences", profile.experiences.len()),
        ("compétences", profile.skills.len()),
        ("formations", profile.education.len()),
        ("langues", profile.languages.len()),
        ("projets", profile.projects.len()),
        ("certifications", profile.certifications.len()),
    ] {
        if len > MAX_ITEMS {
            return Err(AppError::Validation(format!(
                "Le profil contient trop de {label} pour l'assistant IA."
            )));
        }
    }
    Ok(())
}

/// Valide la borne globale d'une valeur structurée reçue de l'IA.
///
/// # Errors
/// Retourne une erreur fournisseur si la sérialisation dépasse la limite.
pub fn validate_structured_size(value: &impl Serialize) -> AppResult<()> {
    let serialized = serde_json::to_string(value)
        .map_err(|error| AppError::Provider(format!("Réponse IA illisible : {error}")))?;
    if serialized.chars().count() > MAX_STRUCTURED_CHARS {
        Err(output_error("la réponse structurée"))
    } else {
        Ok(())
    }
}

/// Refuse une réponse brute excessive avant même la tentative de parsing.
///
/// # Errors
/// Retourne une erreur fournisseur bornée.
pub fn validate_raw_output(raw: &str) -> AppResult<()> {
    if raw.chars().count() > MAX_STRUCTURED_CHARS {
        Err(output_error("la réponse structurée"))
    } else {
        Ok(())
    }
}

impl ValidateAiOutput for StructuredListing {
    fn validate_ai_output(&self) -> AppResult<()> {
        validate_structured_size(self)?;
        ensure_output_string(&self.title, "le titre de l'offre")?;
        for (label, values) in [
            ("les compétences", &self.skills),
            ("les savoir-être", &self.soft_skills),
            ("les mots-clés", &self.keywords),
        ] {
            ensure_output_list(values, label)?;
            validate_strings(values.iter().map(String::as_str), label)?;
        }
        if let Some(experience) = &self.experience {
            ensure_output_string(experience, "l'expérience requise")?;
        }
        Ok(())
    }
}

impl ValidateAiOutput for GeneratedResume {
    fn validate_ai_output(&self) -> AppResult<()> {
        validate_structured_size(self)?;
        ensure_output_string(&self.resume, "le résumé du CV")?;
        ensure_output_list(&self.experiences, "les expériences du CV")?;
        ensure_output_list(&self.skills, "les compétences du CV")?;
        ensure_output_list(&self.education, "les formations du CV")?;
        validate_strings(
            self.skills.iter().map(String::as_str),
            "les compétences du CV",
        )?;
        for experience in &self.experiences {
            validate_strings(
                [
                    experience.title.as_str(),
                    experience.company.as_str(),
                    experience.description.as_str(),
                ],
                "une expérience du CV",
            )?;
        }
        for education in &self.education {
            validate_strings(
                [education.degree.as_str(), education.school.as_str()],
                "une formation du CV",
            )?;
        }
        Ok(())
    }
}

impl ValidateAiOutput for AtsAnalysis {
    fn validate_ai_output(&self) -> AppResult<()> {
        validate_structured_size(self)?;
        ensure_output_string(&self.recap, "le récapitulatif ATS")?;
        ensure_output_list(&self.suggestions, "les suggestions ATS")?;
        ensure_output_list(&self.recommendations, "les recommandations ATS")?;
        validate_strings(
            self.suggestions.iter().map(String::as_str),
            "les suggestions ATS",
        )?;
        for recommendation in &self.recommendations {
            validate_strings(
                [
                    recommendation.section.as_str(),
                    recommendation.original_text.as_str(),
                    recommendation.proposed_text.as_str(),
                ],
                "une recommandation ATS",
            )?;
        }
        Ok(())
    }
}

impl ValidateAiOutput for Profile {
    fn validate_ai_output(&self) -> AppResult<()> {
        validate_structured_size(self)?;
        for (label, len) in [
            ("les expériences du profil", self.experiences.len()),
            ("les compétences du profil", self.skills.len()),
            ("les formations du profil", self.education.len()),
            ("les langues du profil", self.languages.len()),
            ("les projets du profil", self.projects.len()),
            ("les certifications du profil", self.certifications.len()),
        ] {
            if len > MAX_ITEMS {
                return Err(output_error(label));
            }
        }
        let serialized = serde_json::to_value(self)
            .map_err(|error| AppError::Provider(format!("Réponse IA illisible : {error}")))?;
        validate_json_strings(&serialized)?;
        let experience_dates = self.experiences.iter().flat_map(|experience| {
            [
                Some(experience.start_date.as_str()),
                experience.end_date.as_deref(),
            ]
            .into_iter()
            .flatten()
        });
        let education_dates = self.education.iter().flat_map(|education| {
            [
                education.start_date.as_deref(),
                education.end_date.as_deref(),
            ]
            .into_iter()
            .flatten()
        });
        let certification_dates = self
            .certifications
            .iter()
            .filter_map(|certification| certification.date.as_deref());
        for date in experience_dates
            .chain(education_dates)
            .chain(certification_dates)
            .filter(|date| !date.trim().is_empty())
        {
            if !valid_year_or_month(date) {
                return Err(output_error("les dates du profil"));
            }
        }
        Ok(())
    }
}

fn validate_json_strings(value: &serde_json::Value) -> AppResult<()> {
    match value {
        serde_json::Value::String(text) => ensure_output_string(text, "un champ du profil"),
        serde_json::Value::Array(values) => {
            for value in values {
                validate_json_strings(value)?;
            }
            Ok(())
        }
        serde_json::Value::Object(values) => {
            for value in values.values() {
                validate_json_strings(value)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn valid_year_or_month(value: &str) -> bool {
    let bytes = value.as_bytes();
    match bytes {
        [a, b, c, d] => [a, b, c, d].iter().all(|byte| byte.is_ascii_digit()),
        [a, b, c, d, b'-', m1, m2] => {
            [a, b, c, d, m1, m2]
                .iter()
                .all(|byte| byte.is_ascii_digit())
                && value[5..]
                    .parse::<u8>()
                    .is_ok_and(|month| (1..=12).contains(&month))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::errors::AppError;
    use crate::features::ai::domain::{CoverLetterRequest, GeneratedExperience, GeneratedResume};
    use crate::features::profile::domain::{Profile, Skill};

    #[test]
    fn contexte_de_lettre_trop_long_est_refuse() {
        let request = CoverLetterRequest {
            generation_id: "test".into(),
            company: Some("Nova".into()),
            context: Some("x".repeat(10_001)),
            ..CoverLetterRequest::default()
        };

        assert!(matches!(
            validate_cover_letter_request(&request),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn profil_source_trop_volumineux_est_refuse() {
        let mut profile = Profile::default();
        profile.identity.resume = Some("x".repeat(200_001));

        assert!(matches!(
            validate_profile_input(&profile),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn profil_avec_trop_de_competences_est_refuse() {
        let profile = Profile {
            skills: (0..101)
                .map(|index| Skill {
                    name: format!("Compétence {index}"),
                })
                .collect(),
            ..Profile::default()
        };

        assert!(matches!(
            validate_profile_input(&profile),
            Err(AppError::Validation(_))
        ));
    }

    #[test]
    fn description_generee_trop_longue_est_refusee() {
        let resume = GeneratedResume {
            experiences: vec![GeneratedExperience {
                title: "Dev".into(),
                company: "Nova".into(),
                description: "x".repeat(4_001),
            }],
            ..GeneratedResume::default()
        };

        assert!(matches!(
            resume.validate_ai_output(),
            Err(AppError::Provider(_))
        ));
    }

    #[test]
    fn reponse_structuree_trop_volumineuse_est_refusee() {
        let value = serde_json::json!({ "data": "x".repeat(250_001) });

        assert!(matches!(
            validate_structured_size(&value),
            Err(AppError::Provider(_))
        ));
    }
}
