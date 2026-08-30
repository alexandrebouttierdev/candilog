//! Normalisation commune aux comparaisons IA et ATS.

use crate::core::utils::text::search_key;
use std::collections::HashSet;

/// Déduplique des libellés normalisés tout en conservant le premier libellé original.
pub(super) fn deduplicate_labels(values: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .iter()
        .filter_map(|value| {
            let key = search_key(value);
            (!key.is_empty() && seen.insert(key)).then(|| value.clone())
        })
        .collect()
}

/// Recherche un terme normalisé en respectant ses frontières alphanumériques.
pub(super) fn contains_search_term(haystack: &str, needle: &str) -> bool {
    let needle = search_key(needle);
    if needle.is_empty() {
        return false;
    }
    let haystack = search_key(haystack);
    haystack.match_indices(&needle).any(|(index, _)| {
        let before_ok = index == 0
            || !haystack[..index]
                .chars()
                .next_back()
                .is_some_and(char::is_alphanumeric);
        let after = index + needle.len();
        let after_ok = after >= haystack.len()
            || !haystack[after..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric);
        before_ok && after_ok
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplication_conserve_le_premier_libelle() {
        let values = vec!["Café".into(), "cafe".into(), "CAFÉ".into()];
        assert_eq!(deduplicate_labels(&values), vec!["Café"]);
    }
}
