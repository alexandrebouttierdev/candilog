//! Validation d'une source choisie depuis le dialogue natif.

use super::*;

#[test]
fn accepte_uniquement_un_fichier_regulier_avec_extension_autorisee() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("sauvegarde.sqlite");
    std::fs::write(&source, b"sqlite").unwrap();

    assert_eq!(
        validate_selected_source(&source, &["sqlite", "bak"]).unwrap(),
        source.canonicalize().unwrap()
    );
    assert!(validate_selected_source(temp.path(), &["sqlite"]).is_err());
    assert!(validate_selected_source(&source, &["pdf"]).is_err());
}

#[cfg(unix)]
#[test]
fn refuse_une_source_symbolique() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("cv.pdf");
    std::fs::write(&source, b"pdf").unwrap();
    let lien = temp.path().join("lien.pdf");
    symlink(&source, &lien).unwrap();

    assert!(validate_selected_source(&lien, &["pdf"]).is_err());
}
