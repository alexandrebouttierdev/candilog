//! Les compétences du profil restent disponibles sans être copiées dans le CV.

use super::*;

/// La génération ne transforme jamais la bibliothèque complète en section Compétences.
#[test]
fn un_cv_sans_competence_garde_celles_du_profil_en_bibliotheque() {
    let mut generation = generation();
    generation.resume.skills = Vec::new();

    let workspace = prepare_workspace(&profile(), generation, None).unwrap();

    assert!(workspace.document.skill_groups.is_empty());
    assert!(workspace
        .profile_library
        .iter()
        .any(|item| item.label == "Rust"));
}

/// Un profil lui aussi sans compétence ne fabrique pas de section vide.
#[test]
fn un_profil_sans_competence_ne_cree_pas_de_section() {
    let mut generation = generation();
    generation.resume.skills = Vec::new();
    let mut profile = profile();
    profile.skills = Vec::new();

    let workspace = prepare_workspace(&profile, generation, None).unwrap();

    assert!(workspace.document.skill_groups.is_empty());
    assert!(workspace
        .profile_library
        .iter()
        .all(|item| !matches!(item.content, ResumeProfileItemContent::Skill { .. })));
}
