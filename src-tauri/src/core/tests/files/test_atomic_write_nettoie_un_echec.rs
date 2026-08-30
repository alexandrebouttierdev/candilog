//! Une génération échouée ne doit laisser ni cible partielle ni fichier temporaire.

use super::*;

#[test]
fn nettoie_le_temporaire_et_preserve_une_cible_absente() {
    let temp = tempfile::tempdir().unwrap();
    let cible = temp.path().join("cv.pdf");

    let result = atomic_write(&cible, "pdf", |temporaire| {
        std::fs::write(temporaire, b"partiel").unwrap();
        Err(AppError::Validation("document trop long".into()))
    });

    assert!(result.is_err());
    assert!(!cible.exists());
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
}

#[test]
fn publie_le_fichier_uniquement_apres_succes() {
    let temp = tempfile::tempdir().unwrap();
    let cible = temp.path().join("cv.pdf");

    atomic_write(&cible, "pdf", |temporaire| {
        std::fs::write(temporaire, b"document final").unwrap();
        Ok(())
    })
    .unwrap();

    assert_eq!(std::fs::read(cible).unwrap(), b"document final");
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
}
