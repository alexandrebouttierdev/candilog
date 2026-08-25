//! Cas de test isolé.

use super::*;

/// Réenregistrer le poste sans toucher au statut ne doit rien ajouter : chaque étape
/// fictive fausserait l'entonnoir de conversion des analyses.
#[test]
fn test_update_n_historise_que_les_changements_reels() {
    let (repo, entreprise_id) = contexte();
    let creee = repo
        .create(&entree(entreprise_id, "Développeur", "2026-08-20"))
        .unwrap();

    let mut sans_changement = entree(entreprise_id, "Développeur Frontend", "2026-08-20");
    repo.update(creee.id, &sans_changement).unwrap();
    assert_eq!(historique(&repo, creee.id), vec!["EN_ATTENTE"]);

    sans_changement.statut = StatutCandidature::Entretien;
    repo.update(creee.id, &sans_changement).unwrap();
    assert_eq!(historique(&repo, creee.id), vec!["EN_ATTENTE", "ENTRETIEN"]);
}
