//! Cas de test isolé.

use super::*;

#[test]
fn lecture_pdf_refuse_un_fichier_non_pdf() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cv.txt");
    std::fs::write(&path, b"texte").unwrap();
    assert!(read_pdf(&path).is_err());
}
