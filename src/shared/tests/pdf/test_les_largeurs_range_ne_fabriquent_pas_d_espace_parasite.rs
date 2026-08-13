//! Cas de test isolé.

use super::*;

/// Les largeurs d'une police CID déclarées **au format range** (`c_first c_last w`)
/// doivent être respectées : un mot « Développeur » doit s'extraire sans espace parasite
/// entre le « D » et le « é ».
///
/// `pdf-extract` 0.12.0 lisait `c_last` et `c_width` depuis `w[i]` au lieu de `w[i + 1]`
/// et `w[i + 2]` : les largeurs au format range n'étaient jamais insérées, le « D »
/// retombait sur la largeur par défaut `/DW 500`, et l'heuristique d'espace (écart >
/// 10 % de la taille de police) fabriquait « D éveloppeur ». Correctif vendored dans
/// `vendor/pdf-extract-0.12.0` via `[patch.crates-io]`.
#[test]
fn test_les_largeurs_range_ne_fabriquent_pas_d_espace_parasite() {
    let pdf = pdf_avec_largeurs_range();
    let text = extract_text(&pdf).unwrap();
    assert!(
        text.contains("Développeur"),
        "espace parasite entre majuscule et lettre suivante : {text:?}"
    );
    assert!(
        !text.contains("D éveloppeur"),
        "le mot est coupé par un faux espace : {text:?}"
    );
}
