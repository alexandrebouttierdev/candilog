//! Rendu des objets du répertoire d'entreprises.

use crate::modules::entreprises::model::Entreprise;
use crate::ui::format;

pub mod form;

/// Sous-titre d'une entreprise dans une liste : ville, à défaut secteur.
#[must_use]
pub fn subtitle(company: &Entreprise) -> String {
    company
        .ville
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(
            || format::or_else(company.secteur.as_deref(), "Aucune localisation"),
            str::to_owned,
        )
}

/// Détermine si une entreprise correspond à une recherche libre.
#[must_use]
pub fn matches(company: &Entreprise, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    format!(
        "{} {} {}",
        company.nom,
        company.secteur.as_deref().unwrap_or_default(),
        company.ville.as_deref().unwrap_or_default()
    )
    .to_lowercase()
    .contains(needle)
}

#[cfg(test)]
#[path = "tests/mod/mod.rs"]
mod tests;
