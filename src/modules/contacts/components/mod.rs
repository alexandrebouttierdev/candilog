//! Rendu des objets du réseau professionnel.

use crate::modules::contacts::model::Contact;
use crate::ui::format;

/// Nom affichable d'un contact.
#[must_use]
pub fn full_name(contact: &Contact) -> String {
    format!("{} {}", contact.prenom, contact.nom)
        .trim()
        .to_owned()
}

/// Sous-titre d'un contact dans une liste : fonction, à défaut coordonnées.
#[must_use]
pub fn subtitle(contact: &Contact) -> String {
    contact
        .poste
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(
            || format::or_else(contact.email.as_deref(), "Fonction non renseignée"),
            str::to_owned,
        )
}

/// Détermine si un contact correspond à une recherche libre.
#[must_use]
pub fn matches(contact: &Contact, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    format!(
        "{} {} {}",
        full_name(contact),
        contact.poste.as_deref().unwrap_or_default(),
        contact.email.as_deref().unwrap_or_default()
    )
    .to_lowercase()
    .contains(needle)
}

#[cfg(test)]
mod tests {
    use super::{full_name, matches, subtitle};
    use crate::modules::contacts::model::Contact;

    fn contact(poste: Option<&str>, email: Option<&str>) -> Contact {
        Contact {
            id: uuid::Uuid::new_v4(),
            entreprise_id: None,
            prenom: "Alex".into(),
            nom: "Bouttier".into(),
            poste: poste.map(str::to_owned),
            email: email.map(str::to_owned),
            telephone: None,
            linkedin: None,
            notes: None,
            created_at: "2026-08-01".into(),
            updated_at: "2026-08-01".into(),
        }
    }

    #[test]
    fn le_nom_complet_est_normalise() {
        assert_eq!(full_name(&contact(None, None)), "Alex Bouttier");
    }

    #[test]
    fn la_fonction_prime_sur_le_courriel_en_sous_titre() {
        assert_eq!(subtitle(&contact(Some("DRH"), Some("a@b.fr"))), "DRH");
        assert_eq!(subtitle(&contact(None, Some("a@b.fr"))), "a@b.fr");
        assert_eq!(
            subtitle(&contact(Some("  "), None)),
            "Fonction non renseignée"
        );
    }

    #[test]
    fn la_recherche_couvre_nom_fonction_et_courriel() {
        let contact = contact(Some("Responsable RH"), Some("alex@agrial.fr"));
        assert!(matches(&contact, ""));
        assert!(matches(&contact, "bouttier"));
        assert!(matches(&contact, "responsable"));
        assert!(matches(&contact, "agrial"));
        assert!(!matches(&contact, "dupont"));
    }
}
