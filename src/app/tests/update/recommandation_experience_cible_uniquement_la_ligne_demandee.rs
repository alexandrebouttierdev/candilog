//! Cas de test isolé.

use super::*;

#[test]
fn recommandation_experience_cible_uniquement_la_ligne_demandee() {
    let mut cv = GeneratedCv {
        experiences: vec![GeneratedExperience {
            description: "Avant".into(),
            ..GeneratedExperience::default()
        }],
        ..GeneratedCv::default()
    };
    apply_recommendation(
        &mut cv,
        &RecommandationAts {
            section: "experience_0".into(),
            texte_propose: "Après".into(),
            ..RecommandationAts::default()
        },
    );
    assert_eq!(cv.experiences[0].description, "Après");
}
