//! Cas de test isolé.

use super::*;

#[test]
fn test_score_skills_match_partiel_donne_50_et_liste_missing() {
    let p = profile_skills(&["rust"]);
    let o = offer(&["Rust", "Go"], &[], None);
    let s = score(&p, &o);
    assert_eq!(s.skills, 50);
    assert_eq!(s.matched, vec!["Rust"]);
    assert_eq!(s.missing, vec!["Go"]);
}
