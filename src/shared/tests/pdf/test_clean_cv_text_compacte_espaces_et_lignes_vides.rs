//! Cas de test isolé.

use super::*;

#[test]
fn test_clean_cv_text_compacte_espaces_et_lignes_vides() {
    let raw = "Ada   Lovelace\r\n\r\n\r\n\r\nSkills:   Rust,   SQL\r\n";
    assert_eq!(clean_cv_text(raw), "Ada Lovelace\n\nSkills: Rust, SQL");
}
