//! Cas de test isolé.

use super::*;

#[test]
fn l_entreprise_la_plus_tot_est_retournee() {
    let companies = vec![entreprise("Acme Corp"), entreprise("Corp Global")];
    assert_eq!(
        detected_company("Chez Corp Global comme chez Acme Corp.", &companies),
        Some("Corp Global".to_owned())
    );
}
