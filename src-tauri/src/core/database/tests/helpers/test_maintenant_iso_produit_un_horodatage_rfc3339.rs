//! Cas de test isolé.

use super::*;

#[test]
fn test_maintenant_iso_produit_un_horodatage_rfc3339() {
    let timestamp = now_iso();
    assert!(
        chrono::DateTime::parse_from_rfc3339(&timestamp).is_ok(),
        "{timestamp}"
    );
}
