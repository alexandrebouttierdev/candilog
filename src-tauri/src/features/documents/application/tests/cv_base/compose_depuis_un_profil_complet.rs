//! Composition depuis un profil complet, section par section, puis chaque interrupteur
//! d'exclusion pris isolément.

use super::profil_complet;
use crate::features::ai::domain::ProfileSection;
use crate::features::documents::application::resume_workspace::compose_base_resume;

#[test]
fn compose_depuis_un_profil_complet() {
    let document = compose_base_resume(&profil_complet(), &[]).unwrap();

    assert_eq!(document.identity.full_name, "Alex Martin");
    assert_eq!(document.identity.title, "Développeur Rust");
    assert_eq!(document.profile, "Dix ans de back-end.");
    assert_eq!(document.experiences.len(), 1);
    assert_eq!(document.experiences[0].company, "Acme");
    assert_eq!(document.education.len(), 1);
    assert_eq!(document.skill_groups[0].items, vec!["Rust", "SQL"]);
    assert_eq!(document.languages.len(), 1);
}

#[test]
fn une_section_ecartee_ne_figure_pas() {
    let document = compose_base_resume(&profil_complet(), &[ProfileSection::Skills]).unwrap();
    assert!(document.skill_groups.is_empty());
    assert_eq!(document.experiences.len(), 1, "les autres sections restent");
}

#[test]
fn la_presentation_ecartee_vide_le_champ_profile() {
    let document = compose_base_resume(&profil_complet(), &[ProfileSection::Summary]).unwrap();
    assert!(document.profile.is_empty());
}

/// [VIGILANCE 1] Un profil dont tout le contenu est écarté garde son identité.
#[test]
fn un_profil_entierement_ecarte_garde_son_identite() {
    let exclues = [
        ProfileSection::Summary,
        ProfileSection::Experiences,
        ProfileSection::Education,
        ProfileSection::Skills,
        ProfileSection::Languages,
        ProfileSection::Projects,
        ProfileSection::Certifications,
    ];
    let document = compose_base_resume(&profil_complet(), &exclues).unwrap();
    assert_eq!(document.identity.full_name, "Alex Martin");
    assert!(document.experiences.is_empty());
}

/// [VIGILANCE 2] La même compétence saisie deux fois n'apparaît qu'une fois.
#[test]
fn une_competence_en_double_n_apparait_qu_une_fois() {
    let mut profile = profil_complet();
    profile.skills.push(profile.skills[0].clone());
    let document = compose_base_resume(&profile, &[]).unwrap();
    let rust = document.skill_groups[0]
        .items
        .iter()
        .filter(|item| item.eq_ignore_ascii_case("rust"))
        .count();
    assert_eq!(rust, 1);
}

/// [VIGILANCE 3] Une expérience sans date de fin et non courante n'affiche pas de tiret seul.
#[test]
fn une_experience_sans_fin_n_affiche_pas_un_tiret_seul() {
    let mut profile = profil_complet();
    profile.experiences[0].end_date = None;
    profile.experiences[0].current = false;
    let document = compose_base_resume(&profile, &[]).unwrap();
    assert!(!document.experiences[0].period.trim().ends_with('—'));
    assert!(!document.experiences[0].period.trim().is_empty());
}
