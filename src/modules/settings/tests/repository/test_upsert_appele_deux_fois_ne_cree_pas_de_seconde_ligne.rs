//! Cas de test isolé.

use super::*;

#[test]
fn test_upsert_appele_deux_fois_ne_cree_pas_de_seconde_ligne() {
    let repo = repo();
    repo.upsert(&AppSettings::default()).unwrap();
    repo.upsert(&AppSettings::default()).unwrap();
    let conn = crate::shared::sqlite::connexion(&repo.pool).unwrap();
    let lignes: i64 = conn
        .query_row("SELECT count(*) FROM parametres", [], |r| r.get(0))
        .unwrap();
    assert_eq!(lignes, 1);
}
