//! Cas de test isolé.

use super::*;

/// Reposer une carte dans sa colonne d'origine est un geste courant du glisser-déposer :
/// il ne doit laisser aucune trace.
#[test]
fn test_changer_statut_pour_la_meme_valeur_n_ajoute_pas_d_etape() {
    let (repo, company_id) = context();
    let creee = repo
        .create(&entree(company_id, "Développeur", "2026-08-20"))
        .unwrap();

    let apres = repo
        .update_status(creee.id, ApplicationStatus::Pending)
        .unwrap();

    assert_eq!(apres.status, ApplicationStatus::Pending);
    assert_eq!(history(&repo, creee.id), vec!["EN_ATTENTE"]);
}
