//! Cas de test isolé.

use super::*;

#[test]
fn test_score_imported_skills_total_donne_100_et_experience_nulle() {
    let c = cv(&["Rust", "React"], "");
    let o = offer(&["Rust", "React"], &[], Some("5 ans"));
    let s = score_imported(&c, &o);
    assert_eq!(s.skills, 100);
    assert_eq!(s.experience, 0);
    assert_eq!(s.total, 100); // (100*2 + 100)/3 ; mots-clés vides → 100
}
