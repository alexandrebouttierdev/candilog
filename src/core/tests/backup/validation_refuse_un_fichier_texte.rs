//! Cas de test isolé.

use super::*;

#[test]
fn validation_refuse_un_fichier_texte() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("faux.sqlite");
    std::fs::write(&path, b"pas une base").unwrap();
    assert!(validate(&path).is_err());
}
