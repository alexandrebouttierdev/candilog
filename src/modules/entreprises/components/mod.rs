//! Rendu des objets du répertoire d'entreprises.

use crate::modules::entreprises::model::Entreprise;
use crate::ui::format;

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
mod tests {
    use super::{matches, subtitle};
    use crate::modules::entreprises::model::Entreprise;

    fn entreprise(ville: Option<&str>, secteur: Option<&str>) -> Entreprise {
        Entreprise {
            id: uuid::Uuid::new_v4(),
            nom: "Agrial".into(),
            secteur: secteur.map(str::to_owned),
            type_: None,
            site_web: None,
            ville: ville.map(str::to_owned),
            adresse: None,
            notes: None,
            created_at: "2026-08-01".into(),
            updated_at: "2026-08-01".into(),
        }
    }

    #[test]
    fn la_ville_prime_sur_le_secteur_en_sous_titre() {
        assert_eq!(
            subtitle(&entreprise(Some("Rennes"), Some("Agro"))),
            "Rennes"
        );
    }

    #[test]
    fn le_secteur_prend_le_relais_sans_ville() {
        assert_eq!(subtitle(&entreprise(None, Some("Agro"))), "Agro");
        assert_eq!(
            subtitle(&entreprise(Some("   "), None)),
            "Aucune localisation"
        );
    }

    #[test]
    fn la_recherche_couvre_nom_secteur_et_ville() {
        let company = entreprise(Some("Rennes"), Some("Agroalimentaire"));
        assert!(matches(&company, ""));
        assert!(matches(&company, "agrial"));
        assert!(matches(&company, "rennes"));
        assert!(matches(&company, "agroalim"));
        assert!(!matches(&company, "nantes"));
    }
}
