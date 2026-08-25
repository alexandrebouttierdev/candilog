//! Cas de test isolé.

use super::*;

/// Reposer une carte dans sa colonne d'origine est un geste courant du glisser-déposer :
/// il ne doit laisser aucune trace.
#[test]
fn test_changer_statut_pour_la_meme_valeur_n_ajoute_pas_d_etape() {
    let (repo, entreprise_id) = contexte();
    let creee = repo
        .create(&entree(entreprise_id, "Développeur", "2026-08-20"))
        .unwrap();

    let apres = repo
        .update_statut(creee.id, StatutCandidature::EnAttente)
        .unwrap();

    assert_eq!(apres.statut, StatutCandidature::EnAttente);
    assert_eq!(historique(&repo, creee.id), vec!["EN_ATTENTE"]);
}
