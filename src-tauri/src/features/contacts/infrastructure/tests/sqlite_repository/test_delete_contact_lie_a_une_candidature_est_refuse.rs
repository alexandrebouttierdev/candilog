//! Cas de test isolé.

use super::*;

#[test]
fn test_delete_contact_lie_a_une_candidature_est_refuse() {
    let repo = repo();
    let cree = repo.create(&entree("Bouttier", None)).unwrap();
    candidature_liee(&repo, cree.id);
    let resultat = repo.delete(cree.id);
    match resultat {
        Err(AppError::Validation(message)) => {
            assert!(message.contains("Suppression impossible"), "{message}");
        }
        autre => panic!("attendu Validation, obtenu {autre:?}"),
    }
    assert_eq!(
        repo.list().unwrap().len(),
        1,
        "le contact a été supprimé malgré le refus"
    );
}
