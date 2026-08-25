//! Cas de test isolé.

use super::*;

#[test]
fn test_page_vide_conserve_une_page_totale() {
    // Le pied de pagination afficherait « page 1 sur 0 » si le total tombait à zéro.
    assert_eq!(Page::new(Vec::<u64>::new(), 0, 1, 8).total_pages, 1);
}
