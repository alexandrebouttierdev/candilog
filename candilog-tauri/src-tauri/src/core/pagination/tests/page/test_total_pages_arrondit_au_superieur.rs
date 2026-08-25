//! Cas de test isolé.

use super::*;

#[test]
fn test_total_pages_arrondit_au_superieur() {
    // 15 éléments par pages de 8 font deux pages, pas une : arrondir vers le bas cacherait
    // les sept derniers éléments derrière un pied de pagination qui n'en propose aucune.
    assert_eq!(Page::new(Vec::<u64>::new(), 15, 1, 8).total_pages, 2);
}
