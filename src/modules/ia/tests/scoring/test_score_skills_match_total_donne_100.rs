//! Cas de test isolé.

use super::*;

#[test]
fn test_score_skills_match_total_donne_100() {
    let p = profile_skills(&["Rust", "React"]);
    let o = offer(&["Rust", "React"], &[], None);
    let s = score(&p, &o);
    assert_eq!(s.skills, 100);
    assert_eq!(s.matched.len(), 2);
    assert!(s.missing.is_empty());
}
