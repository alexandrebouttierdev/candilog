//! Cas de test isolé.

use super::*;

#[test]
fn test_lister_appels_page_borne_et_compte_le_total() {
    let r = repo();
    for index in 0..25 {
        r.enregistrer_appel(&appel(
            OperationLlm::ParseOffer,
            &format!("2026-07-16T10:{index:02}:00Z"),
            true,
        ))
        .unwrap();
    }
    let page = r.lister_appels_page(2, 10).unwrap();
    assert_eq!(page.items.len(), 10);
    assert_eq!(page.total, 25);
    assert_eq!(page.total_pages, 3);
    assert_eq!(page.page, 2);
}
