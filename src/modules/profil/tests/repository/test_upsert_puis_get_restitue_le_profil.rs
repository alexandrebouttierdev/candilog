//! Cas de test isolé.

use super::*;

#[test]
fn test_upsert_puis_get_restitue_le_profil() {
    let repo = repo();
    let mut profil = Profile::default();
    profil.personal.first_name = "Alexandre".into();
    profil.personal.last_name = "Bouttier".into();
    repo.upsert(&profil).unwrap();
    let relu = repo.get().unwrap();
    assert_eq!(relu.personal.first_name, "Alexandre");
    assert_eq!(relu.personal.last_name, "Bouttier");
}
