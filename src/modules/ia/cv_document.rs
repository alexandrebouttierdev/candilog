//! Fusion du profil et du CV généré en un modèle unique, partagé par l'aperçu
//! Iced et l'export PDF : les deux rendus reposent ainsi exactement sur les
//! mêmes données.

use crate::core::cv_pdf::{CvEducation, CvExperience, CvLanguage, CvPdf, CvProject};
use crate::modules::ia::cv_model::CvGeneration;
use crate::shared::profile::Profile;

/// Construit le modèle de CV, en fusionnant le profil (identité, coordonnées,
/// projets, langues, périodes) et le CV généré (contenu reformulé).
#[must_use]
pub fn construire(profile: &Profile, generation: &CvGeneration) -> CvPdf {
    let personnel = &profile.personal;
    let mut cv = CvPdf {
        name: format!("{} {}", personnel.first_name, personnel.last_name)
            .trim()
            .to_owned(),
        subtitle: personnel.headline.clone().unwrap_or_default(),
        phone: personnel.phone.clone(),
        email: personnel.email.clone(),
        city: personnel.city.clone(),
        linkedin: personnel.linkedin.clone(),
        website: personnel.website.clone(),
        profil: generation.cv.summary.clone(),
        skills: generation.cv.skills.clone(),
        ..CvPdf::default()
    };

    cv.experiences = generation
        .cv
        .experiences
        .iter()
        .map(|experience| {
            let meta = profile
                .experiences
                .iter()
                .find(|e| e.title.trim() == experience.title.trim())
                .map(|e| {
                    let mut parties = Vec::new();
                    if let Some(lieu) = e.location.as_deref() {
                        if !lieu.trim().is_empty() {
                            parties.push(lieu.to_owned());
                        }
                    }
                    let periode = formater_periode(Some(&e.start_date), e.end_date.as_deref());
                    if !periode.is_empty() {
                        parties.push(periode);
                    }
                    parties.join(" · ")
                })
                .unwrap_or_default();
            CvExperience {
                title: experience.title.clone(),
                company: experience.company.clone(),
                meta,
                bullets: decouper_puces(&experience.description),
            }
        })
        .collect();

    cv.projects = profile
        .projects
        .iter()
        .map(|projet| CvProject {
            name: projet.name.clone(),
            meta: projet.technologies.clone().unwrap_or_default(),
            bullets: projet
                .description
                .as_deref()
                .map(decouper_puces)
                .unwrap_or_default(),
        })
        .collect();

    cv.education = generation
        .cv
        .education
        .iter()
        .map(|education| {
            let date = profile
                .education
                .iter()
                .find(|e| e.degree.trim() == education.degree.trim())
                .map(|e| formater_periode(e.start_date.as_deref(), e.end_date.as_deref()))
                .unwrap_or_default();
            CvEducation {
                degree: education.degree.clone(),
                school: education.school.clone(),
                date,
            }
        })
        .collect();

    cv.languages = profile
        .languages
        .iter()
        .map(|langue| CvLanguage {
            name: langue.name.clone(),
            level: langue.level.clone(),
        })
        .collect();

    cv
}

/// Découpe une description en puces : une ligne non vide = une puce, les
/// marqueurs courants (`·`, `-`, `•`) étant retirés en tête de ligne.
fn decouper_puces(description: &str) -> Vec<String> {
    description
        .lines()
        .map(|ligne| {
            ligne
                .trim()
                .trim_start_matches(['·', '-', '•', '*', ' '])
                .trim()
                .to_owned()
        })
        .filter(|ligne| !ligne.is_empty())
        .collect()
}

/// Formate une période « début – fin » en français.
fn formater_periode(debut: Option<&str>, fin: Option<&str>) -> String {
    match (debut, fin) {
        (Some(debut), Some(fin)) => {
            format!(
                "{} – {}",
                formater_date_mois(debut),
                formater_date_mois(fin)
            )
        }
        (Some(debut), None) => formater_date_mois(debut),
        (None, Some(fin)) => formater_date_mois(fin),
        (None, None) => String::new(),
    }
}

/// Formate une date `AAAA-MM` en « Mois. AAAA », ou `AAAA` telle quelle.
fn formater_date_mois(valeur: &str) -> String {
    let Some((annee, mois)) = valeur.split_once('-') else {
        return valeur.to_owned();
    };
    let (Ok(annee), Ok(mois)) = (annee.parse::<u32>(), mois.parse::<u32>()) else {
        return valeur.to_owned();
    };
    format!("{} {annee}", mois_abrege(mois))
}

/// Abréviation française d'un mois, de 1 à 12.
const fn mois_abrege(numero: u32) -> &'static str {
    match numero {
        1 => "Janv.",
        2 => "Févr.",
        3 => "Mars",
        4 => "Avr.",
        5 => "Mai",
        6 => "Juin",
        7 => "Juil.",
        8 => "Août",
        9 => "Sept.",
        10 => "Oct.",
        11 => "Nov.",
        12 => "Déc.",
        _ => "?",
    }
}

#[cfg(test)]
#[path = "tests/cv_document/mod.rs"]
mod tests;
