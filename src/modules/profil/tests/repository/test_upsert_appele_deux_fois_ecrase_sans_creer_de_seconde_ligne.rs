//! Cas de test isolé.

use super::*;

#[test]
fn test_upsert_appele_deux_fois_ecrase_sans_creer_de_seconde_ligne() {
    let repo = repo();
    let mut premier = Profile::default();
    premier.personal.last_name = "Premier".into();
    repo.upsert(&premier).unwrap();
    let mut second = Profile::default();
    second.personal.last_name = "Second".into();
    repo.upsert(&second).unwrap();
    assert_eq!(repo.get().unwrap().personal.last_name, "Second");
    let conn = crate::shared::sqlite::connexion(&repo.pool).unwrap();
    let lignes: i64 = conn
        .query_row("SELECT count(*) FROM profil", [], |r| r.get(0))
        .unwrap();
    assert_eq!(lignes, 1);
}
