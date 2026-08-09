//! Cas de test isolé.

use super::*;

#[test]
fn test_enregistrer_score_puis_lister_restitue_les_champs() {
    let r = repo();
    r.enregistrer_score(&ScoreAts {
        score: 82,
        origine: OrigineScore::Genere,
        cree_le: "2026-07-16T10:00:00Z".into(),
    })
    .unwrap();
    let scores = r.lister_scores().unwrap();
    assert_eq!(scores.len(), 1);
    assert_eq!(scores[0].score, 82);
    assert_eq!(scores[0].origine, OrigineScore::Genere);
}
