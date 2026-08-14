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
#[path = "tests/mod/mod.rs"]
mod tests;
