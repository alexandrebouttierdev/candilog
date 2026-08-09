//! Cas de test isolé.

use super::*;

#[test]
fn test_clean_cv_text_ne_supprime_aucun_contenu() {
    let raw = "  Ligne 1  \n\n  Ligne 2  ";
    let cleaned = clean_cv_text(raw);
    assert!(cleaned.contains("Ligne 1"));
    assert!(cleaned.contains("Ligne 2"));
}
