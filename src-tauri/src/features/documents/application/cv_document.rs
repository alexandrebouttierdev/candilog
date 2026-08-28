//! Fusion du profil et du CV généré pour l'export PDF.

use crate::features::ia::domain::CvGenere;
use crate::features::profil::domain::Profil;
use crate::infrastructure::pdf::{CvEducation, CvExperience, CvLanguage, CvPdf, CvProject};

/// Construit le modèle de CV, en fusionnant le profil (identité, coordonnées,
/// projets, langues, périodes) et le CV généré (contenu reformulé).
#[must_use]
pub fn construire(profil: &Profil, generation: &CvGenere) -> CvPdf {
    let identite = &profil.identite;
    let mut cv = CvPdf {
        name: format!("{} {}", identite.prenom, identite.nom)
            .trim()
            .to_owned(),
        subtitle: identite.titre.clone().unwrap_or_default(),
        phone: identite.telephone.clone(),
        email: identite.email.clone(),
        city: identite.ville.clone(),
        linkedin: identite.linkedin.clone(),
        website: identite.site_web.clone(),
        profil: generation.resume.clone(),
        skills: generation.competences.clone(),
        ..CvPdf::default()
    };

    cv.experiences = generation
        .experiences
        .iter()
        .map(|experience| {
            let meta = profil
                .experiences
                .iter()
                .find(|e| e.intitule.trim() == experience.intitule.trim())
                .map(|e| {
                    let mut parties = Vec::new();
                    if let Some(lieu) = e.lieu.as_deref() {
                        if !lieu.trim().is_empty() {
                            parties.push(lieu.to_owned());
                        }
                    }
                    let periode = formater_periode(Some(&e.date_debut), e.date_fin.as_deref());
                    if !periode.is_empty() {
                        parties.push(periode);
                    }
                    parties.join(" · ")
                })
                .unwrap_or_default();
            CvExperience {
                title: experience.intitule.clone(),
                company: experience.entreprise.clone(),
                meta,
                bullets: decouper_puces(&experience.description),
            }
        })
        .collect();

    cv.projects = profil
        .projets
        .iter()
        .map(|projet| CvProject {
            name: projet.nom.clone(),
            meta: projet.technologies.clone().unwrap_or_default(),
            bullets: projet
                .description
                .as_deref()
                .map(decouper_puces)
                .unwrap_or_default(),
        })
        .collect();

    cv.education = generation
        .formations
        .iter()
        .map(|education| {
            let date = profil
                .formations
                .iter()
                .find(|e| e.diplome.trim() == education.diplome.trim())
                .map(|e| formater_periode(e.date_debut.as_deref(), e.date_fin.as_deref()))
                .unwrap_or_default();
            CvEducation {
                degree: education.diplome.clone(),
                school: education.etablissement.clone(),
                date,
            }
        })
        .collect();

    cv.languages = profil
        .langues
        .iter()
        .map(|langue| CvLanguage {
            name: langue.nom.clone(),
            level: langue.niveau.clone(),
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
