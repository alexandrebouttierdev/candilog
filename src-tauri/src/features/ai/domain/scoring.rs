//! Score ATS local et déterministe, sans appel réseau.

use super::{GeneratedResume, StructuredListing, MatchScore};
use crate::features::profile::domain::Profile;
use chrono::Datelike;

#[must_use]
pub fn profile_score(profile: &Profile, job_offer: &StructuredListing) -> MatchScore {
    let names: Vec<String> = profile
        .skills
        .iter()
        .map(|c| c.name.to_lowercase())
        .collect();
    let (present, missing): (Vec<_>, Vec<_>) = job_offer
        .skills
        .iter()
        .cloned()
        .partition(|c| names.contains(&c.to_lowercase()));
    let skills = percentage(present.len(), job_offer.skills.len());
    let text = format!(
        "{} {} {}",
        profile.identity.title.as_deref().unwrap_or_default(),
        profile.identity.resume.as_deref().unwrap_or_default(),
        profile
            .experiences
            .iter()
            .map(|e| format!(
                "{} {}",
                e.title,
                e.description.as_deref().unwrap_or_default()
            ))
            .collect::<Vec<_>>()
            .join(" ")
    )
    .to_lowercase();
    let key = job_offer
        .keywords
        .iter()
        .filter(|m| text.contains(&m.to_lowercase()))
        .count();
    let ats = percentage(key, job_offer.keywords.len());
    let requis = job_offer.experience.as_deref().map_or(0, first_entier);
    let current = chrono::Utc::now().year();
    let annees: usize = profile
        .experiences
        .iter()
        .filter_map(|e| {
            year(&e.start_date).map(|start| {
                (year(e.end_date.as_deref().unwrap_or_default()).unwrap_or(current) - start).max(0)
                    as usize
            })
        })
        .sum();
    let experience = annees
        .saturating_mul(100)
        .checked_div(requis)
        .map_or(100, |v| v.min(100) as u8);
    let total =
        ((u16::from(skills) * 40 + u16::from(experience) * 40 + u16::from(ats) * 20 + 50)
            / 100) as u8;
    MatchScore {
        total,
        skills,
        experience,
        ats,
        present,
        missing,
    }
}

#[must_use]
pub fn score_resume_imported(resume: &GeneratedResume, job_offer: &StructuredListing) -> MatchScore {
    let names: Vec<String> = resume.skills.iter().map(|c| c.to_lowercase()).collect();
    let (present, missing): (Vec<_>, Vec<_>) = job_offer
        .skills
        .iter()
        .cloned()
        .partition(|c| names.contains(&c.to_lowercase()));
    let skills = percentage(present.len(), job_offer.skills.len());
    let text = serde_json::to_string(resume).unwrap_or_default().to_lowercase();
    let key = job_offer
        .keywords
        .iter()
        .filter(|m| text.contains(&m.to_lowercase()))
        .count();
    let ats = percentage(key, job_offer.keywords.len());
    MatchScore {
        total: ((u16::from(skills) * 2 + u16::from(ats)) / 3) as u8,
        skills,
        experience: 0,
        ats,
        present,
        missing,
    }
}

fn percentage(count: usize, total: usize) -> u8 {
    count
        .saturating_mul(100)
        .checked_div(total)
        .map_or(100, |v| v.min(100) as u8)
}
fn first_entier(value: &str) -> usize {
    value
        .split(|c: char| !c.is_ascii_digit())
        .find(|v| !v.is_empty())
        .and_then(|v| v.parse().ok())
        .unwrap_or_default()
}
fn year(value: &str) -> Option<i32> {
    value
        .as_bytes()
        .windows(4)
        .find_map(|v| std::str::from_utf8(v).ok()?.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::profile::domain::{Skill, Experience, Identity, Profile};

    fn profile_rust() -> Profile {
        Profile {
            identity: Identity {
                first_name: "Camille".into(),
                name: "Martin".into(),
                email: "camille@example.fr".into(),
                phone: None,
                city: None,
                title: Some("Développeuse Rust".into()),
                resume: Some("Systèmes et CLI".into()),
                linkedin: None,
                github: None,
                website: None,
            },
            experiences: vec![Experience {
                title: "Ingénieure".into(),
                company: "Nova".into(),
                location: None,
                start_date: "2020-01".into(),
                end_date: None,
                current: true,
                description: Some("APIs Rust".into()),
            }],
            skills: vec![Skill { name: "Rust".into() }],
            education: vec![],
            languages: vec![],
            projects: vec![],
            certifications: vec![],
        }
    }

    #[test]
    fn une_competence_presente_augmente_le_score() {
        let job_offer = StructuredListing {
            title: "Rust".into(),
            skills: vec!["Rust".into(), "React".into()],
            soft_skills: vec![],
            experience: Some("3 ans".into()),
            keywords: vec!["cli".into(), "kubernetes".into()],
        };
        let score = profile_score(&profile_rust(), &job_offer);
        assert_eq!(score.present, vec!["Rust"]);
        assert_eq!(score.missing, vec!["React"]);
        assert_eq!(score.skills, 50);
        assert!(score.total > 0);
    }
}
