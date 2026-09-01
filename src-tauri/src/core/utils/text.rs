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

/// Colle une préposition à un mot en respectant l'élision : « de Astek » → « d'Astek ».
///
/// Les documents générés composent leurs phrases autour de valeurs saisies par
/// l'utilisateur — un poste, une entreprise. Sans élision, une lettre de motivation
/// s'ouvrait sur « ma candidature au poste de Administrateur », faute que personne ne
/// commettrait dans une candidature réelle.
///
/// Le `h` est traité comme muet : le distinguer du `h` aspiré demanderait un lexique, et
/// « d'hôpital » est correct quand « de hall » est rare dans un intitulé de poste.
#[must_use]
pub fn elider(preposition: &str, suivant: &str) -> String {
    let suivant = suivant.trim();
    let premiere = suivant
        .nfd()
        .find(|character| !is_combining_mark(*character))
        .map(|character| character.to_ascii_lowercase());
    if matches!(premiere, Some('a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'h')) {
        let radical = preposition.trim_end_matches(['e', 'a']);
        format!("{radical}’{suivant}")
    } else {
        format!("{preposition} {suivant}")
    }
}

/// Découpe un mot à ses traits d'union, chaque fragment gardant le sien.
///
/// C'est l'occasion de césure que le navigateur prend en premier : « Jean-Baptiste » passe
/// à la ligne après le trait d'union, pas au milieu de « Baptiste ». Les moteurs PDF
/// n'avaient que la coupe caractère par caractère, et l'aperçu et la page imprimée
/// cassaient les mêmes noms à deux endroits différents.
#[must_use]
pub fn segments_de_cesure(token: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut debut = 0;
    for (index, caractere) in token.char_indices() {
        if caractere == '-' && index + caractere.len_utf8() < token.len() {
            segments.push(&token[debut..=index]);
            debut = index + caractere.len_utf8();
        }
    }
    if debut < token.len() {
        segments.push(&token[debut..]);
    }
    if segments.is_empty() {
        segments.push(token);
    }
    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalise_accents_casse_et_espaces() {
        assert_eq!(search_key("  CAFÉ\tCrème  "), "cafe creme");
    }

    #[test]
    fn elide_devant_une_voyelle_accentuee_ou_non() {
        assert_eq!(elider("de", "Astek"), "d’Astek");
        assert_eq!(elider("de", "Éditions Lumen"), "d’Éditions Lumen");
        assert_eq!(elider("de", "Hôtellerie Réunie"), "d’Hôtellerie Réunie");
        assert_eq!(elider("de", "  Ingénieur  "), "d’Ingénieur");
    }

    #[test]
    fn coupe_un_mot_a_ses_traits_d_union() {
        assert_eq!(
            segments_de_cesure("Jean-Baptiste"),
            vec!["Jean-", "Baptiste"]
        );
        assert_eq!(
            segments_de_cesure("Maréchal-de-Lattre-de-Tassigny"),
            vec!["Maréchal-", "de-", "Lattre-", "de-", "Tassigny"]
        );
        assert_eq!(segments_de_cesure("Vandenberghe"), vec!["Vandenberghe"]);
        // Un trait d'union final n'ouvre pas de fragment vide.
        assert_eq!(segments_de_cesure("Nord-"), vec!["Nord-"]);
        assert_eq!(segments_de_cesure(""), vec![""]);
    }

    #[test]
    fn n_elide_pas_devant_une_consonne() {
        assert_eq!(elider("de", "Technicien"), "de Technicien");
        assert_eq!(elider("de", "Ville de Rennes"), "de Ville de Rennes");
    }

    #[test]
    fn normalise_une_majuscule_accentuee_comme_sa_minuscule() {
        assert_eq!(search_key("ÉCOLE DIRECTE"), search_key("école directe"));
    }
}
