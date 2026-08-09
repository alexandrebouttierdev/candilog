//! Cas de test isolé.

use super::*;

#[test]
fn test_score_imported_match_partiel_liste_matched_et_missing() {
    let c = cv(&["rust"], "");
    let o = offer(&["Rust", "Go"], &[], None);
    let s = score_imported(&c, &o);
    assert_eq!(s.skills, 50);
    assert_eq!(s.matched, vec!["Rust"]);
    assert_eq!(s.missing, vec!["Go"]);
}
