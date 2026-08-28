//! Cas de test isolé.

use super::*;

/// L'historique est ce qui permet de compter les candidatures **passées** par l'entretien,
/// y compris refusées ensuite. Sans étape initiale, une candidature créée directement au
/// statut « Entretien » serait invisible de l'entonnoir de conversion.
#[test]
fn test_create_ouvre_l_historique_de_statut() {
    let (repo, entreprise_id) = contexte();
    let creee = repo
        .create(&entree(entreprise_id, "Développeur", "2026-08-20"))
        .unwrap();

    assert_eq!(historique(&repo, creee.id), vec!["EN_ATTENTE"]);
}
