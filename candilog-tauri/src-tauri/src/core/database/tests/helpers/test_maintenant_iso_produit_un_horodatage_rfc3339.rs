//! Cas de test isolé.

use super::*;

#[test]
fn test_maintenant_iso_produit_un_horodatage_rfc3339() {
    let horodatage = maintenant_iso();
    assert!(
        chrono::DateTime::parse_from_rfc3339(&horodatage).is_ok(),
        "{horodatage}"
    );
}
