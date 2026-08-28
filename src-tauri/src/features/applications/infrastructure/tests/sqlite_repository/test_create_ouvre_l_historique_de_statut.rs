//! Cas de test isolé.

use super::*;

/// L'historique est ce qui permet de compter les candidatures **passées** par l'entretien,
/// y compris refusées ensuite. Sans étape initiale, une candidature créée directement au
/// statut « Interview » serait invisible de l'entonnoir de conversion.
#[test]
fn test_create_ouvre_l_historique_de_statut() {
    let (repo, company_id) = context();
    let creee = repo
        .create(&entree(company_id, "Développeur", "2026-08-20"))
        .unwrap();

    assert_eq!(history(&repo, creee.id), vec!["EN_ATTENTE"]);
}
