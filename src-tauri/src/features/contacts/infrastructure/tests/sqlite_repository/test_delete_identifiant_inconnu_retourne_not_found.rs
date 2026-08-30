use super::*;

#[test]
fn delete_identifiant_inconnu_retourne_not_found() {
    let repo = repo();
    assert!(matches!(
        repo.delete(uuid::Uuid::new_v4()),
        Err(AppError::NotFound(_))
    ));
}
