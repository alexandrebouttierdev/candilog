//! Composition du document autonome à partir des instantanés métier.

use super::*;

#[test]
fn compose_un_document_autonome() {
    let mut source = profile();
    let workspace = prepare_workspace(&source, generation()).unwrap();

    assert_eq!(workspace.schema_version, 1);
    assert_eq!(workspace.document.identity.full_name, "Alex Exemple");
    assert_eq!(
        workspace.document.profile,
        "Développeur orienté produit et qualité."
    );
    assert_eq!(
        workspace.document.experiences[0].period,
        "Janv. 2024 — Aujourd’hui"
    );
    assert_eq!(workspace.document.projects[0].name, "Candilog");
    assert_eq!(workspace.document.skill_groups[0].name, "Compétences");
    assert_eq!(workspace.initial_score, workspace.score.total);
    assert_ne!(workspace.initial_score, 99);
    // L'offre attend "SQL", absent du CV généré : une proposition de compétence manquante
    // est déjà construite à la composition, pour que l'IPC de la tâche 3 la retourne sans
    // recalcul supplémentaire.
    assert_eq!(workspace.proposals.len(), 1);
    assert_eq!(
        workspace.proposals[0].kind,
        ResumeProposalKind::MissingSkill
    );
    assert_eq!(workspace.proposals[0].proposed_text, "SQL");

    let snapshot = workspace.clone();
    source.identity.first_name = "Camille".into();
    source.projects[0].name = "Projet modifié".into();
    assert_eq!(workspace, snapshot);
}

#[test]
fn reconstruit_un_cv_genere_depuis_le_document() {
    let workspace = prepare_workspace(&profile(), generation()).unwrap();
    let generated = to_generated_resume(&workspace.document);

    assert_eq!(generated.resume, workspace.document.profile);
    assert_eq!(generated.skills, vec!["Rust", "React"]);
    assert_eq!(
        generated.experiences[0].description,
        "Conception d'une application\nTests automatisés"
    );
    assert_eq!(generated.education[0].degree, "TSSR");
}

#[test]
fn normalise_les_liens_historiques_sans_schema() {
    let mut source = profile();
    source.identity.linkedin = Some("linkedin.com/in/alex".into());
    source.identity.website = Some("alex.example.test".into());
    source.projects[0].url = Some("project.example.test/demo".into());

    let workspace = prepare_workspace(&source, generation()).unwrap();

    assert_eq!(
        workspace.document.identity.linkedin.as_deref(),
        Some("https://linkedin.com/in/alex")
    );
    assert_eq!(
        workspace.document.identity.website.as_deref(),
        Some("https://alex.example.test/")
    );
    assert_eq!(
        workspace.document.projects[0].url.as_deref(),
        Some("https://project.example.test/demo")
    );
}

#[test]
fn ne_normalise_pas_un_chemin_relatif_en_domaine() {
    let mut source = profile();
    source.identity.website = Some("/profil".into());

    assert!(matches!(
        prepare_workspace(&source, generation()),
        Err(AppError::Validation(message)) if message == "Le site web du CV doit être une URL valide"
    ));
}
