//! Cas de test isolé.

use super::*;

/// Un `None` doit produire une cellule vide, jamais la chaîne « None » : le fichier est lu
/// par un humain dans un tableur.
#[test]
fn test_les_champs_absents_deviennent_vides() {
    let mut sans_company = cand("Développeur", None);
    sans_company.company_name = None;
    sans_company.effective_city = None;
    sans_company.effective_address = None;
    sans_company.professional_domain_name = None;
    sans_company.effective_company_type_name = None;
    sans_company.weekly_hours = None;

    let csv = vers_csv(&[sans_company]).unwrap();

    assert!(csv.contains("Développeur;;Offre d'emploi;CDI;Temps plein;;;;PME;;;"));
    assert!(!csv.contains("None"));
}

/// Sans libellé résolu, le code du contrat vaut mieux qu'une cellule vide : la ligne reste
/// exploitable, et l'absence signalerait à tort un contrat non renseigné.
#[test]
fn le_code_du_contrat_supplee_un_libelle_non_resolu() {
    let mut sans_libelle = cand("Développeur", None);
    sans_libelle.contract_type_name = None;
    sans_libelle.contract_type_code = "MIS".into();

    let csv = vers_csv(&[sans_libelle]).unwrap();

    assert!(csv.contains(";MIS;"));
}
