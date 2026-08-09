//! Cas de test isolé.

use super::*;

#[test]
fn test_extract_text_pdf_valide_retourne_le_texte() {
    let pdf = pdf_avec_texte("Ingenieur Rust");
    let text = extract_text(&pdf).unwrap();
    assert!(
        text.contains("Ingenieur"),
        "texte extrait inattendu : {text:?}"
    );
}
