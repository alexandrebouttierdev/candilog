//! Rendu des objets du réseau professionnel.

use crate::modules::contacts::model::Contact;
use crate::ui::components::{avatar, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{button, column, container, row};
use iced::{Alignment, Element, Length};

/// Nom affichable d'un contact.
#[must_use]
pub fn full_name(contact: &Contact) -> String {
    format!("{} {}", contact.prenom, contact.nom)
        .trim()
        .to_owned()
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

/// Carte de contact : avatar accent, nom, poste, e-mail en pied.
pub fn contact_card<'a, Message: Clone + 'a>(
    contact: &Contact,
    on_press: Message,
) -> Element<'a, Message> {
    let name = full_name(contact);
    let body = container(
        column![
            row![
                avatar::avatar(avatar::initials_of(&name), 40.0, Tone::Accent),
                column![
                    typo::item(name),
                    typo::caption(format::or_dash(contact.poste.as_deref())),
                ]
                .spacing(2.0),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
            surface::divider(),
            typo::text_mono(
                format::or_dash(contact.email.as_deref()),
                font::CAPTION,
                font::MONO_REGULAR,
            )
            .style(styles::muted_text),
        ]
        .spacing(space::MD),
    )
    .padding(space::LG)
    .width(Length::Fill)
    .style(styles::glass_card);

    button(body)
        .width(Length::Fill)
        .style(styles::card)
        .on_press(on_press)
        .into()
}

#[cfg(test)]
mod tests {
    use super::{full_name, matches};
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
    fn la_recherche_couvre_nom_fonction_et_courriel() {
        let contact = contact(Some("Responsable RH"), Some("alex@agrial.fr"));
        assert!(matches(&contact, ""));
        assert!(matches(&contact, "bouttier"));
        assert!(matches(&contact, "responsable"));
        assert!(matches(&contact, "agrial"));
        assert!(!matches(&contact, "dupont"));
    }

    #[test]
    fn la_carte_de_contact_s_instancie_avec_et_sans_coordonnees() {
        use super::contact_card;
        use iced::Element;

        let bare = contact(None, None);
        let _: Element<'_, ()> = contact_card(&bare, ());
        let complete = contact(Some("DRH"), Some("a@b.fr"));
        let _: Element<'_, ()> = contact_card(&complete, ());
    }
}
