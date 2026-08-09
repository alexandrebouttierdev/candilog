//! Cas de test isolé.

use super::*;

#[test]
fn test_update_statut_modifie_uniquement_le_statut() {
    let repo = repo();
    let ent = entreprise(&repo, "ACME");
    let creee = repo.create(&entree(ent, "Dev")).unwrap();
    let modifiee = repo
        .update_statut(creee.id, StatutCandidature::Refus)
        .unwrap();
    assert_eq!(modifiee.statut, StatutCandidature::Refus);
    assert_eq!(modifiee.poste, "Dev");
}
