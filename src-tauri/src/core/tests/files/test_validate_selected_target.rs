//! Validation d'une destination choisie depuis le dialogue natif.

use super::*;

#[test]
fn refuse_un_chemin_relatif_et_une_mauvaise_extension() {
    assert!(validate_selected_target(std::path::Path::new("cv.pdf"), "pdf").is_err());

    let temp = tempfile::tempdir().unwrap();
    assert!(validate_selected_target(&temp.path().join("cv.txt"), "pdf").is_err());
}

#[cfg(unix)]
#[test]
fn refuse_un_lien_symbolique_existant() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let fichier = temp.path().join("document.pdf");
    std::fs::write(&fichier, b"contenu").unwrap();
    let lien = temp.path().join("lien.pdf");
    symlink(&fichier, &lien).unwrap();

    assert!(validate_selected_target(&lien, "pdf").is_err());
}

#[test]
fn normalise_le_parent_sans_modifier_le_nom() {
    let temp = tempfile::tempdir().unwrap();
    let cible = temp.path().join("CV.PDF");

    let validee = validate_selected_target(&cible, "pdf").unwrap();

    assert_eq!(validee, temp.path().canonicalize().unwrap().join("CV.PDF"));
}
