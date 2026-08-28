//! Cas de test isolé.

use super::*;

#[test]
fn test_le_poste_et_l_entreprise_sont_aplatis() {
    let (repo, candidature_id) = contexte();

    let creee = repo.create(&entree(candidature_id, "2026-08-27")).unwrap();

    assert_eq!(creee.candidature_poste.as_deref(), Some("Product Designer"));
    assert_eq!(creee.entreprise_nom.as_deref(), Some("Atlas Studio"));
}
