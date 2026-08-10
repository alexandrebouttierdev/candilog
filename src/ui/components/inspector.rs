//! Inspecteur desktop : sections, lignes de propriété, actions.
//!
//! Aucune propriété n'est encadrée individuellement. La lecture se fait en
//! balayant une colonne d'étiquettes alignées.

use super::typo;
use crate::ui::theme::metrics::{size, space};
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, Space};
use iced::{Alignment, Element, Length};

/// Ligne de propriété : étiquette à gauche, valeur alignée à droite.
pub fn property<'a, Message: 'a>(label: &'a str, value: impl Into<String>) -> Element<'a, Message> {
    container(
        row![
            typo::meta(label),
            Space::with_width(Length::Fill),
            typo::body(value.into()),
        ]
        .spacing(space::LG)
        .align_y(Alignment::Center),
    )
    .height(size::ROW)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

/// Ligne de propriété dont la valeur porte un ton sémantique.
pub fn property_toned<'a, Message: 'a>(
    label: &'a str,
    value: impl Into<String>,
    tone: Tone,
) -> Element<'a, Message> {
    container(
        row![
            typo::meta(label),
            Space::with_width(Length::Fill),
            typo::toned(value.into(), tone),
        ]
        .spacing(space::LG)
        .align_y(Alignment::Center),
    )
    .height(size::ROW)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

/// Ligne de propriété portant un contrôle plutôt qu'une valeur.
pub fn property_control<'a, Message: 'a>(
    label: &'a str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        row![
            typo::meta(label),
            Space::with_width(Length::Fill),
            control.into(),
        ]
        .spacing(space::LG)
        .align_y(Alignment::Center),
    )
    .height(size::ROW)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

/// Groupe de propriétés précédé d'un intitulé discret : une section.
pub fn group<'a, Message: 'a>(
    title: &'a str,
    rows: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut body = column![].spacing(0);
    for entry in rows {
        body = body.push(entry);
    }
    column![typo::label(title), super::surface::divider(), body,]
        .spacing(space::XL)
        .width(Length::Fill)
        .into()
}

/// Bloc de texte long d'un inspecteur (notes, résumé).
pub fn note<'a, Message: 'a>(title: &'a str, content: Option<String>) -> Element<'a, Message> {
    let body: Element<'a, Message> = match content.filter(|value| !value.trim().is_empty()) {
        Some(value) => typo::body(value).into(),
        None => typo::caption("Aucune note enregistrée.").into(),
    };
    column![typo::label(title), super::surface::divider(), body]
        .spacing(space::SM)
        .width(Length::Fill)
        .into()
}

/// Barre d'actions d'un inspecteur, alignée à droite.
pub fn actions<'a, Message: 'a>(
    controls: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut line = row![Space::with_width(Length::Fill)]
        .spacing(space::SM)
        .align_y(Alignment::Center);
    for control in controls {
        line = line.push(control);
    }
    line.into()
}

// Les lignes de propriété partagent la hauteur des lignes de table
// (`size::ROW`) : une seule échelle de densité pour toute donnée alignée en
// colonne, table ou inspecteur. Le plancher de cette hauteur est garanti par
// l'invariant central `size::ROW >= 40.0` dans `theme::metrics`.
