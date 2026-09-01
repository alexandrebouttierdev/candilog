//! Cas de test isolé.

use super::*;

/// Une génération sans compétence ne doit pas amputer le CV.
///
/// La validation de sortie ne borne qu'un maximum : une liste vide passait, et le CV
/// partait sans sa section Compétences alors que le profil en portait.
#[test]
fn un_cv_sans_competence_reprend_celles_du_profil() {
    let mut generation = generation();
    generation.resume.skills = Vec::new();

    let workspace = prepare_workspace(&profile(), generation).unwrap();

    let groupes = &workspace.document.skill_groups;
    assert_eq!(groupes.len(), 1, "section Compétences absente du CV");
    assert_eq!(groupes[0].items, vec!["Rust".to_owned()]);
}

/// Un profil lui aussi sans compétence ne fabrique pas de section vide.
#[test]
fn un_profil_sans_competence_ne_cree_pas_de_section() {
    let mut generation = generation();
    generation.resume.skills = Vec::new();
    let mut profile = profile();
    profile.skills = Vec::new();

    let workspace = prepare_workspace(&profile, generation).unwrap();

    assert!(workspace.document.skill_groups.is_empty());
}
