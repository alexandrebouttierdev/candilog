//! Cas de test isolé.

use super::*;

#[test]
fn test_score_offre_sans_skills_ne_divise_pas_par_zero() {
    let p = profile_skills(&["Rust"]);
    let o = offer(&[], &[], None);
    let s = score(&p, &o);
    assert_eq!(s.skills, 0);
}
