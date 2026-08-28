//! Cas de test isolé.

use super::*;

#[test]
fn test_offset_de_la_premiere_page_est_nul() {
    assert_eq!(Page::<u64>::offset(1, 8), 0);
    assert_eq!(Page::<u64>::offset(3, 8), 16);
    // Une page 0 remontée par un appelant fautif ne doit pas déborder en arithmétique
    // non signée.
    assert_eq!(Page::<u64>::offset(0, 8), 0);
}
