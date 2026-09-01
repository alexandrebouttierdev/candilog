//! Cas de test isolé.

use super::*;

/// Une génération sans formation ne doit pas amputer le CV.
///
/// Le recadrage sur les faits (`ground_generated_resume`) écarte toute formation que le
/// modèle n'a pas recopiée à l'identique : reformuler « BTS SIO » en « BTS Services
/// informatiques aux organisations » vidait la section, et le CV s'exportait sans sa
/// formation sans que rien ne le signale.
#[test]
fn un_cv_sans_formation_reprend_celles_du_profil() {
    let mut generation = generation();
    generation.resume.education = Vec::new();

    let workspace = prepare_workspace(&profile(), generation).unwrap();

    let formations = &workspace.document.education;
    assert_eq!(formations.len(), 1, "section Formation absente du CV");
    assert_eq!(formations[0].degree, "TSSR");
    assert_eq!(formations[0].school, "ENI");
    assert!(
        !formations[0].period.trim().is_empty(),
        "la période du profil doit suivre la formation reprise"
    );
}

/// Même règle pour les expériences : sans repli, le CV partait sans aucun parcours.
#[test]
fn un_cv_sans_experience_reprend_celles_du_profil() {
    let mut generation = generation();
    generation.resume.experiences = Vec::new();

    let workspace = prepare_workspace(&profile(), generation).unwrap();

    let experiences = &workspace.document.experiences;
    assert_eq!(experiences.len(), 1, "section Expériences absente du CV");
    assert_eq!(experiences[0].company, "Exemple SARL");
    assert_eq!(
        experiences[0].bullets,
        vec!["Description source".to_owned()]
    );
    assert!(!experiences[0].period.trim().is_empty());
}

/// Un profil lui aussi vide ne fabrique pas de section creuse.
#[test]
fn un_profil_sans_formation_ni_experience_ne_cree_pas_de_section() {
    let mut generation = generation();
    generation.resume.education = Vec::new();
    generation.resume.experiences = Vec::new();
    let mut profile = profile();
    profile.education = Vec::new();
    profile.experiences = Vec::new();

    let workspace = prepare_workspace(&profile, generation).unwrap();

    assert!(workspace.document.education.is_empty());
    assert!(workspace.document.experiences.is_empty());
}

/// Le cas le plus fréquent n'est pas la liste vide mais la liste amputée : le modèle
/// reformule un seul diplôme, le recadrage l'écarte, et le CV part avec un diplôme de moins.
#[test]
fn une_formation_ecartee_par_le_recadrage_est_retablie() {
    let mut profile = profile();
    profile.education.push(Education {
        degree: "Licence professionnelle ASUR".into(),
        school: "IUT de Rennes".into(),
        location: None,
        start_date: Some("2023-09".into()),
        end_date: Some("2024-06".into()),
        description: None,
    });

    // La génération n'en renvoie qu'une : l'autre a été reformulée puis écartée.
    let workspace = prepare_workspace(&profile, generation()).unwrap();

    let diplomes: Vec<_> = workspace
        .document
        .education
        .iter()
        .map(|entree| entree.degree.as_str())
        .collect();
    assert_eq!(diplomes, vec!["TSSR", "Licence professionnelle ASUR"]);
}

/// L'ordre choisi par le modèle est la part d'adaptation à l'offre : il passe en premier,
/// ce qu'il a omis vient ensuite, dans l'ordre du profil.
#[test]
fn l_ordre_de_la_generation_est_conserve() {
    let mut profile = profile();
    profile.experiences.insert(
        0,
        Experience {
            title: "Alternant".into(),
            company: "Ancienne SARL".into(),
            start_date: "2022-09".into(),
            end_date: Some("2023-12".into()),
            ..Experience::default()
        },
    );

    let workspace = prepare_workspace(&profile, generation()).unwrap();

    let entreprises: Vec<_> = workspace
        .document
        .experiences
        .iter()
        .map(|entree| entree.company.as_str())
        .collect();
    assert_eq!(entreprises, vec!["Exemple SARL", "Ancienne SARL"]);
}
