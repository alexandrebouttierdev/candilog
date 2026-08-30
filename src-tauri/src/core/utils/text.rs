//! Normalisation de texte partagée par la recherche SQL et les comparaisons IA.

use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

/// Produit une clé de recherche insensible à la casse, aux accents et aux espaces répétés.
///
/// Placée dans `core` et non dans une feature : la même normalisation doit s'appliquer des
/// deux côtés de la comparaison — au terme saisi, en Rust, et à la colonne, dans SQLite via
/// la fonction scalaire `search_key`. `lower()` de SQLite n'agit que sur l'ASCII : normaliser
/// d'un seul côté rendait « ÉCOLE » introuvable, y compris en le cherchant par son nom exact.
#[must_use]
pub fn search_key(value: &str) -> String {
    value
        .nfd()
        .filter(|character| !is_combining_mark(*character))
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalise_accents_casse_et_espaces() {
        assert_eq!(search_key("  CAFÉ\tCrème  "), "cafe creme");
    }

    #[test]
    fn normalise_une_majuscule_accentuee_comme_sa_minuscule() {
        assert_eq!(search_key("ÉCOLE DIRECTE"), search_key("école directe"));
    }
}
