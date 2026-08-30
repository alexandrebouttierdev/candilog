use super::*;

#[test]
fn delete_identifiant_inconnu_retourne_not_found() {
    let (repo, _) = context();
    assert!(matches!(
        repo.delete(Uuid::new_v4()),
        Err(AppError::NotFound(_))
    ));
}
