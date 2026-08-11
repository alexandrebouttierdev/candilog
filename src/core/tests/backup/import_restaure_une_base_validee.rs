//! Cas de test isolé.

use super::*;

#[test]
fn import_restaure_une_base_validee() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("backup.sqlite");
    let source = open_pool(None).unwrap();
    run_local_migrations(&source).unwrap();
    export(&source, &path).unwrap();
    let target = open_pool(None).unwrap();
    run_local_migrations(&target).unwrap();
    import(&target, std::path::Path::new(""), &path).unwrap();
    validate(&path).unwrap();
}
