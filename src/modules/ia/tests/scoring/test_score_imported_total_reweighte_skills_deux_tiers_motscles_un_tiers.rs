//! Cas de test isolé.

use super::*;

#[test]
fn test_score_imported_total_reweighte_skills_deux_tiers_motscles_un_tiers() {
    // skills 0 %, mots-clés 100 % → total = (0*2 + 100)/3 = 33.
    let c = cv(&[], "expert kubernetes terraform");
    let o = offer(&["Rust"], &["kubernetes", "terraform"], None);
    let s = score_imported(&c, &o);
    assert_eq!(s.skills, 0);
    assert_eq!(s.ats, 100);
    assert_eq!(s.total, 33);
}
