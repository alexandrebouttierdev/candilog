//! Sélection volontaire, priorisation et contrainte de mise en page réelle.

use super::*;

fn profile_with_skill_count(count: usize) -> Profile {
    let mut value = profile();
    value.skills = (0..count)
        .map(|index| Skill {
            name: format!("Compétence {index}"),
        })
        .collect();
    value
}

#[test]
fn trois_ou_trente_competences_restent_dans_la_bibliotheque() {
    for count in [3, 30] {
        let profile = profile_with_skill_count(count);
        let workspace = prepare_workspace(&profile, generation(), None).unwrap();

        assert!(workspace.document.skill_groups.is_empty());
        assert_eq!(
            workspace
                .profile_library
                .iter()
                .filter(|item| matches!(item.content, ResumeProfileItemContent::Skill { .. }))
                .count(),
            count
        );
    }
}

#[test]
fn nombreux_projets_et_certifications_ne_sont_pas_copies() {
    let mut profile = profile();
    profile.projects = (0..18)
        .map(|index| Project {
            name: format!("Projet {index}"),
            description: Some("Résultat concret".into()),
            ..Project::default()
        })
        .collect();
    profile.certifications = (0..14)
        .map(|index| Certification {
            name: format!("Certification {index}"),
            ..Certification::default()
        })
        .collect();

    let workspace = prepare_workspace(&profile, generation(), None).unwrap();

    assert!(workspace.document.projects.is_empty());
    assert!(workspace.document.certifications.is_empty());
    assert!(workspace.document.languages.is_empty());
    assert_eq!(workspace.profile_library.len(), 18 + 14 + 1 + 1);
}

#[test]
fn un_profil_sans_contenu_optionnel_produit_un_socle_exploitable() {
    let mut profile = profile();
    profile.skills.clear();
    profile.projects.clear();
    profile.certifications.clear();
    profile.languages.clear();
    let mut generation = generation();
    generation.analysis.content_recommendations.clear();

    let workspace = prepare_workspace(&profile, generation, None).unwrap();

    assert!(!workspace.document.experiences.is_empty());
    assert!(!workspace.document.education.is_empty());
    assert!(workspace.profile_library.is_empty());
    assert!(workspace.content_recommendations.is_empty());
}

#[test]
fn beaucoup_de_candidates_sont_priorisees_et_bornees_a_quatre() {
    let profile = profile_with_skill_count(12);
    let mut generation = generation();
    generation.analysis.content_recommendations = (0..8)
        .map(|index| AtsContentRecommendation {
            item_id: format!("skill-{index}"),
            reason: format!("Pertinence {index}"),
            relevance: if index == 6 {
                ContentRelevance::VeryRelevant
            } else {
                ContentRelevance::Relevant
            },
        })
        .collect();

    let workspace = prepare_workspace(&profile, generation, None).unwrap();

    assert!(workspace.content_recommendations.len() <= 4);
    assert_eq!(workspace.content_recommendations[0].label, "Compétence 6");
}

#[test]
fn une_decision_ignoree_ou_un_retrait_explicite_ne_revient_pas_au_recalcul() {
    let profile = profile_with_skill_count(3);
    let mut generation = generation();
    generation.analysis.content_recommendations = vec![
        AtsContentRecommendation {
            item_id: "skill-0".into(),
            reason: "Prioritaire".into(),
            relevance: ContentRelevance::VeryRelevant,
        },
        AtsContentRecommendation {
            item_id: "skill-1".into(),
            reason: "Utile".into(),
            relevance: ContentRelevance::Relevant,
        },
    ];
    let mut workspace = prepare_workspace(&profile, generation, None).unwrap();
    workspace.decisions.ignored.push("skill-0".into());
    workspace
        .decisions
        .explicitly_removed
        .push("skill-1".into());

    let recalculated = recalculate(workspace, None).unwrap();

    assert!(recalculated.content_recommendations.is_empty());
}

#[test]
fn un_ajout_qui_provoque_un_overflow_n_est_pas_recommande() {
    let mut profile = profile();
    profile.skills.clear();
    profile.certifications.clear();
    profile.languages.clear();
    profile.projects = vec![Project {
        name: "Projet trop détaillé".into(),
        description: Some("Réalisation technique avec résultat. ".repeat(650)),
        ..Project::default()
    }];
    let mut generation = generation();
    generation.analysis.content_recommendations = vec![AtsContentRecommendation {
        item_id: "project-0".into(),
        reason: "Projet pertinent".into(),
        relevance: ContentRelevance::VeryRelevant,
    }];

    let workspace = prepare_workspace(&profile, generation, None).unwrap();

    assert!(workspace.content_recommendations.is_empty());
    assert!(workspace.document.projects.is_empty());
    assert!(workspace
        .profile_library
        .iter()
        .any(|item| item.id == "project-0"));
}

#[test]
fn la_mesure_pdf_distingue_un_cv_aere_d_un_debordement() {
    let short = minimal_document();
    let short_layout = measure(&short, None).unwrap();
    assert_eq!(
        short_layout.status,
        crate::features::documents::domain::ResumeLayoutStatus::Spacious
    );
    assert!(!short_layout.overflow);

    for (repeat, expected) in [
        (
            60,
            crate::features::documents::domain::ResumeLayoutStatus::Available,
        ),
        (
            80,
            crate::features::documents::domain::ResumeLayoutStatus::AlmostFull,
        ),
        (
            110,
            crate::features::documents::domain::ResumeLayoutStatus::Full,
        ),
    ] {
        let mut sampled = minimal_document();
        sampled.profile = "Profil technique détaillé avec un résultat mesurable. ".repeat(repeat);
        assert_eq!(measure(&sampled, None).unwrap().status, expected);
    }

    let mut long = short;
    long.profile = "Profil technique détaillé avec un résultat mesurable. ".repeat(400);
    let long_layout = measure(&long, None).unwrap();
    assert_eq!(
        long_layout.status,
        crate::features::documents::domain::ResumeLayoutStatus::Overflow
    );
    assert!(long_layout.overflow);
    assert!(long_layout.remaining_points < 0);
}

#[test]
fn un_remplacement_est_propose_sans_modifier_le_document() {
    let mut profile = profile();
    profile.skills = vec![Skill {
        name: "Active Directory".into(),
    }];
    profile.projects = vec![Project {
        name: "Projet secondaire".into(),
        description: Some(
            (0..75)
                .map(|index| format!("Détail secondaire {index}"))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        ..Project::default()
    }];
    profile.certifications.clear();
    profile.languages.clear();
    let mut generation = generation();
    generation.job_offer.skills = vec!["Active Directory".into()];
    generation.analysis.content_recommendations = vec![AtsContentRecommendation {
        item_id: "skill-0".into(),
        reason: "Directement demandé dans l'offre.".into(),
        relevance: ContentRelevance::VeryRelevant,
    }];
    let mut workspace = prepare_workspace(&profile, generation, None).unwrap();
    let project = workspace
        .profile_library
        .iter()
        .find(|item| matches!(item.content, ResumeProfileItemContent::Project { .. }))
        .unwrap()
        .clone();
    insert_profile_item(&mut workspace.document, &project);

    let recalculated = recalculate(workspace, None).unwrap();

    assert_eq!(
        recalculated.document.projects.len(),
        1,
        "le remplacement n'est jamais automatique"
    );
    assert!(recalculated.document.skill_groups.is_empty());
    assert!(recalculated
        .content_recommendations
        .iter()
        .any(|recommendation| {
            matches!(
                recommendation.action,
                ResumeContentRecommendationAction::Replace { .. }
            )
        }));
}
