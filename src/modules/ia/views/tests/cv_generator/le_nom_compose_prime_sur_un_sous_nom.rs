//! Cas de test isolé.

use super::*;

#[test]
fn le_nom_compose_prime_sur_un_sous_nom() {
    let companies = vec![entreprise("Corp"), entreprise("Corp Global")];
    assert_eq!(
        detected_company("Rejoignez Corp Global.", &companies),
        Some("Corp Global".to_owned())
    );
}
