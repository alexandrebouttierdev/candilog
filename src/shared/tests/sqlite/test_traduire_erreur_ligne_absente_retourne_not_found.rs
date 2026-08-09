//! Cas de test isolé.

use super::*;

#[test]
fn test_traduire_erreur_ligne_absente_retourne_not_found() {
    let traduite = traduire_erreur(rusqlite::Error::QueryReturnedNoRows, "candidature");
    assert!(matches!(traduite, AppError::NotFound(_)));
}
