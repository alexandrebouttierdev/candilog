//! Cas de test isolé.

use super::*;

#[test]
fn test_reset_scores_vide_le_journal() {
    let r = repo();
    r.enregistrer_score(&ScoreAts {
        score: 50,
        origine: OrigineScore::Importe,
        cree_le: "2026-07-16T10:00:00Z".into(),
    })
    .unwrap();
    r.reset_scores().unwrap();
    assert!(r.lister_scores().unwrap().is_empty());
}
