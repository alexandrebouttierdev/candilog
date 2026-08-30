//! Une base non vide sans version n'est pas une base neuve sûre à initialiser.

use super::*;

#[test]
fn base_version_zero_non_vide_est_refusee() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("candilog.sqlite");
    {
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection
            .execute_batch("CREATE TABLE candidatures(id TEXT PRIMARY KEY);")
            .unwrap();
    }
    let before = std::fs::read(&path).unwrap();

    let error = validate_database_file(&path).unwrap_err();

    assert!(matches!(error, AppError::IncompatibleData(_)));
    assert_eq!(std::fs::read(&path).unwrap(), before);
}
