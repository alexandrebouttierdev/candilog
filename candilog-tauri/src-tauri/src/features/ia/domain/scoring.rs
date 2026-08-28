//! Score ATS local et déterministe, sans appel réseau.

use super::{CvGenere, OffreStructuree, ScoreCorrespondance};
use crate::features::profil::domain::Profil;
use chrono::Datelike;

#[must_use]
pub fn score_profil(profil: &Profil, offre: &OffreStructuree) -> ScoreCorrespondance {
    let noms: Vec<String> = profil
        .competences
        .iter()
        .map(|c| c.nom.to_lowercase())
        .collect();
    let (presentes, absentes): (Vec<_>, Vec<_>) = offre
        .competences
        .iter()
        .cloned()
        .partition(|c| noms.contains(&c.to_lowercase()));
    let competences = pourcentage(presentes.len(), offre.competences.len());
    let texte = format!(
        "{} {} {}",
        profil.identite.titre.as_deref().unwrap_or_default(),
        profil.identite.resume.as_deref().unwrap_or_default(),
        profil
            .experiences
            .iter()
            .map(|e| format!(
                "{} {}",
                e.intitule,
                e.description.as_deref().unwrap_or_default()
            ))
            .collect::<Vec<_>>()
            .join(" ")
    )
    .to_lowercase();
    let mots = offre
        .mots_cles
        .iter()
        .filter(|m| texte.contains(&m.to_lowercase()))
        .count();
    let ats = pourcentage(mots, offre.mots_cles.len());
    let requis = offre.experience.as_deref().map_or(0, premier_entier);
    let actuel = chrono::Utc::now().year();
    let annees: usize = profil
        .experiences
        .iter()
        .filter_map(|e| {
            annee(&e.date_debut).map(|debut| {
                (annee(e.date_fin.as_deref().unwrap_or_default()).unwrap_or(actuel) - debut).max(0)
                    as usize
            })
        })
        .sum();
    let experience = annees
        .saturating_mul(100)
        .checked_div(requis)
        .map_or(100, |v| v.min(100) as u8);
    let total =
        ((u16::from(competences) * 40 + u16::from(experience) * 40 + u16::from(ats) * 20 + 50)
            / 100) as u8;
    ScoreCorrespondance {
        total,
        competences,
        experience,
        ats,
        presentes,
        absentes,
    }
}

#[must_use]
pub fn score_cv_importe(cv: &CvGenere, offre: &OffreStructuree) -> ScoreCorrespondance {
    let noms: Vec<String> = cv.competences.iter().map(|c| c.to_lowercase()).collect();
    let (presentes, absentes): (Vec<_>, Vec<_>) = offre
        .competences
        .iter()
        .cloned()
        .partition(|c| noms.contains(&c.to_lowercase()));
    let competences = pourcentage(presentes.len(), offre.competences.len());
    let texte = serde_json::to_string(cv).unwrap_or_default().to_lowercase();
    let mots = offre
        .mots_cles
        .iter()
        .filter(|m| texte.contains(&m.to_lowercase()))
        .count();
    let ats = pourcentage(mots, offre.mots_cles.len());
    ScoreCorrespondance {
        total: ((u16::from(competences) * 2 + u16::from(ats)) / 3) as u8,
        competences,
        experience: 0,
        ats,
        presentes,
        absentes,
    }
}

fn pourcentage(nombre: usize, total: usize) -> u8 {
    nombre
        .saturating_mul(100)
        .checked_div(total)
        .map_or(100, |v| v.min(100) as u8)
}
fn premier_entier(value: &str) -> usize {
    value
        .split(|c: char| !c.is_ascii_digit())
        .find(|v| !v.is_empty())
        .and_then(|v| v.parse().ok())
        .unwrap_or_default()
}
fn annee(value: &str) -> Option<i32> {
    value
        .as_bytes()
        .windows(4)
        .find_map(|v| std::str::from_utf8(v).ok()?.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::profil::domain::{Competence, Experience, Identite, Profil};

    fn profil_rust() -> Profil {
        Profil {
            identite: Identite {
                prenom: "Camille".into(),
                nom: "Martin".into(),
                email: "camille@example.fr".into(),
                telephone: None,
                ville: None,
                titre: Some("Développeuse Rust".into()),
                resume: Some("Systèmes et CLI".into()),
                linkedin: None,
                github: None,
                site_web: None,
            },
            experiences: vec![Experience {
                intitule: "Ingénieure".into(),
                entreprise: "Nova".into(),
                lieu: None,
                date_debut: "2020-01".into(),
                date_fin: None,
                poste_actuel: true,
                description: Some("APIs Rust".into()),
            }],
            competences: vec![Competence { nom: "Rust".into() }],
            formations: vec![],
            langues: vec![],
            projets: vec![],
            certifications: vec![],
        }
    }

    #[test]
    fn une_competence_presente_augmente_le_score() {
        let offre = OffreStructuree {
            titre: "Rust".into(),
            competences: vec!["Rust".into(), "React".into()],
            savoir_etre: vec![],
            experience: Some("3 ans".into()),
            mots_cles: vec!["cli".into(), "kubernetes".into()],
        };
        let score = score_profil(&profil_rust(), &offre);
        assert_eq!(score.presentes, vec!["Rust"]);
        assert_eq!(score.absentes, vec!["React"]);
        assert_eq!(score.competences, 50);
        assert!(score.total > 0);
    }
}
