//! Composants de la bibliothèque de lettres.

use crate::modules::lettres::model::LettreMotivation;

/// Recherche sur le nom, l'entreprise et le poste, insensible à la casse.
#[must_use]
pub fn matches(letter: &LettreMotivation, needle: &str) -> bool {
    needle.is_empty()
        || letter.name.to_lowercase().contains(needle)
        || letter
            .company
            .as_deref()
            .unwrap_or_default()
            .to_lowercase()
            .contains(needle)
        || letter
            .job_title
            .as_deref()
            .unwrap_or_default()
            .to_lowercase()
            .contains(needle)
}
