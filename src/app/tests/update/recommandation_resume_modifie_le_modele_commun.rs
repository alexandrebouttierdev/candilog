//! Cas de test isolé.

use super::*;

#[test]
fn recommandation_resume_modifie_le_modele_commun() {
    let mut cv = GeneratedCv {
        summary: "Avant".into(),
        ..GeneratedCv::default()
    };
    apply_recommendation(
        &mut cv,
        &RecommandationAts {
            section: "resume".into(),
            texte_propose: "Après".into(),
            ..RecommandationAts::default()
        },
    );
    assert_eq!(cv.summary, "Après");
}
