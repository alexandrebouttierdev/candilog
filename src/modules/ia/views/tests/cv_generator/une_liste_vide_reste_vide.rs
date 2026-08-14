//! Cas de test isolé.

use super::*;

#[test]
fn une_liste_vide_reste_vide() {
    let analysis = score(&[], &[]);
    assert!(present_skills(&analysis).is_empty());
    assert!(missing_skills(&analysis).is_empty());
}
