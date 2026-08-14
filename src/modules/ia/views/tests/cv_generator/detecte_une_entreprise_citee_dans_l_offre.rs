//! Cas de test isolé.

use super::*;

#[test]
fn detecte_une_entreprise_citee_dans_l_offre() {
    let companies = vec![entreprise("Acme Corp"), entreprise("Globex")];
    assert_eq!(
        detected_company("Rejoignez Acme Corp à Paris.", &companies),
        Some("Acme Corp".to_owned())
    );
}
