//! Cas de test isolé.

use super::*;

#[test]
fn test_taille_de_page_nulle_est_ramenee_a_un() {
    // Une taille nulle traverserait l'IPC depuis un appelant fautif et ferait diviser
    // par zéro le calcul du nombre de pages.
    let page = Page::new(Vec::<u64>::new(), 4, 1, 0);
    assert_eq!(page.page_size, 1);
    assert_eq!(page.total_pages, 4);
}
