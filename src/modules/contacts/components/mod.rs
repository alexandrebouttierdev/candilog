//! Rendu des objets du réseau professionnel.

use crate::modules::contacts::model::Contact;
use crate::ui::components::icon::{self, Icon};
use crate::ui::components::{avatar, layout, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{button, column, row};
use iced::{Alignment, Element, Length};

pub mod form;

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
    let body = column![
        row![
            avatar::avatar(avatar::initials_of(&name), 44.0, Tone::Accent),
            column![
                typo::item(name),
                typo::caption(format::or_else(
                    contact.poste.as_deref(),
                    "Fonction non renseignée",
                )),
            ]
            .spacing(space::XXS),
            layout::spacer(),
            icon::muted(Icon::ChevronRight),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
        surface::divider(),
        row![
            icon::muted(Icon::Mail),
            typo::caption(format::or_else(
                contact.email.as_deref(),
                "Aucun e-mail renseigné",
            )),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
    ]
    .spacing(space::MD);

    button(body)
        .width(Length::Fill)
        .padding(space::LG)
        .style(styles::contact_card)
        .on_press(on_press)
        .into()
}

#[cfg(test)]
#[path = "tests/mod/mod.rs"]
mod tests;
