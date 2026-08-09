//! Cas de test isolé.

use super::*;

#[test]
fn test_score_experience_aucune_exigence_donne_100() {
    let p = Profile::default();
    let o = offer(&[], &[], None);
    assert_eq!(score(&p, &o).experience, 100);
}
