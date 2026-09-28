//! Sections du profil que l'utilisateur retire d'une génération (« Ce que l'IA peut
//! utiliser », « Arguments autorisés »).

use crate::features::profile::domain::Profile;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Section du profil. Une section exclue n'est ni envoyée au modèle, ni reprise dans le
/// document, ni proposée ensuite comme contenu à ajouter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum ProfileSection {
    /// Le résumé de l'identité (« Présentation »).
    Summary,
    /// La disponibilité déclarée dans l'identité.
    Availability,
    Experiences,
    Education,
    Skills,
    Languages,
    Projects,
    Certifications,
    Interests,
}

/// Copie du profil sans les sections exclues. L'identité (nom, coordonnées, titre) reste :
/// elle en-tête le document et ne constitue pas un argument.
#[must_use]
pub fn profile_without(profile: &Profile, excluded: &[ProfileSection]) -> Profile {
    let mut kept = profile.clone();
    for section in excluded {
        match section {
            ProfileSection::Summary => kept.identity.resume = None,
            ProfileSection::Availability => kept.identity.availability = None,
            ProfileSection::Experiences => kept.experiences.clear(),
            ProfileSection::Education => kept.education.clear(),
            ProfileSection::Skills => kept.skills.clear(),
            ProfileSection::Languages => kept.languages.clear(),
            ProfileSection::Projects => kept.projects.clear(),
            ProfileSection::Certifications => kept.certifications.clear(),
            ProfileSection::Interests => kept.interests.clear(),
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::profile::domain::{Education, Experience, Skill};

    fn profile() -> Profile {
        let mut profile = Profile::default();
        profile.identity.first_name = "Jean".into();
        profile.identity.resume = Some("Chargé d'exploitation".into());
        profile.experiences.push(Experience {
            title: "Technicien".into(),
            company: "Ker Informatique".into(),
            ..Experience::default()
        });
        profile.education.push(Education {
            degree: "BTS SIO".into(),
            school: "Lycée Chateaubriand".into(),
            ..Education::default()
        });
        profile.skills.push(Skill {
            name: "Linux".into(),
            ..Skill::default()
        });
        profile
    }

    #[test]
    fn une_section_exclue_disparait_et_les_autres_restent() {
        let kept = profile_without(
            &profile(),
            &[ProfileSection::Skills, ProfileSection::Summary],
        );

        assert!(kept.skills.is_empty());
        assert!(kept.identity.resume.is_none());
        assert_eq!(kept.experiences.len(), 1);
        assert_eq!(kept.education.len(), 1);
        assert_eq!(kept.identity.first_name, "Jean");
    }

    #[test]
    fn sans_exclusion_le_profil_est_intact() {
        assert_eq!(profile_without(&profile(), &[]), profile());
    }
}
