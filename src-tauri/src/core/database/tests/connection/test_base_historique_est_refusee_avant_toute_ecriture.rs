//! Une base historique doit être identifiée avant toute ouverture en écriture.

use super::*;

#[test]
fn base_historique_est_refusee_avant_toute_ecriture() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("candilog.sqlite");
    {
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE candidatures(id TEXT PRIMARY KEY);
                 INSERT INTO candidatures(id) VALUES ('historique');
                 PRAGMA user_version = 9;",
            )
            .unwrap();
    }
    let before = std::fs::read(&path).unwrap();

    let error = validate_database_file(&path).unwrap_err();

    assert!(matches!(error, AppError::IncompatibleData(_)));
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn chemin_absent_est_accepte_comme_base_neuve() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("nouvelle.sqlite");

    validate_database_file(&path).unwrap();

    assert!(!path.exists());
}
