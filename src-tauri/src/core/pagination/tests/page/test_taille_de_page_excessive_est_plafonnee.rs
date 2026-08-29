//! Cas de test isolé.

use super::*;

#[test]
fn test_taille_de_page_excessive_est_plafonnee() {
    let page = Page::new(Vec::<u64>::new(), 50_000, 1, u64::MAX);
    assert_eq!(page.page_size, MAX_PAGE_SIZE);
}
