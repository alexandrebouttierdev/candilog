//! Le CSV est relu hors de l'application : il porte les valeurs effectives, pas les
//! surcharges brutes.

use super::*;

#[test]
fn une_valeur_heritee_est_exportee_comme_une_surcharge() {
    // Ville et adresse héritées de l'entreprise : les colonnes du fichier doivent les
    // porter, sinon un export de candidatures sans surcharge sortirait sans localisation.
    let heritee = cand("Développeur", None);
    assert!(heritee.city.is_none());

    let csv = vers_csv(&[heritee]).unwrap();

    assert!(csv.contains(";Rennes;12 rue des Lilas;"));
    assert!(csv.contains("ESN / Société de services numériques"));
}

#[test]
fn la_surcharge_prime_dans_le_fichier() {
    let mut surchargee = cand("Développeur", None);
    surchargee.city = Some("Nantes".into());
    surchargee.effective_city = Some("Nantes".into());
    surchargee.effective_company_type_id = Some("FINAL_CLIENT".into());
    surchargee.effective_company_type_name = Some("Client final".into());

    let csv = vers_csv(&[surchargee]).unwrap();

    assert!(csv.contains("Client final"));
    assert!(csv.contains(";Nantes;"));
    assert!(!csv.contains("Rennes"));
}
