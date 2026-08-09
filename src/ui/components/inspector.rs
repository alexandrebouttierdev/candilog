//! Inspecteur desktop : sections, lignes de propriété, actions.
//!
//! Aucune propriété n'est encadrée individuellement. La lecture se fait en
//! balayant une colonne d'étiquettes alignées.

use super::typo;
use crate::ui::theme::metrics::space;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, Space};
use iced::{Alignment, Element, Length};

/// Hauteur d'une ligne de propriété.
const PROPERTY_ROW: f32 = 26.0;

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
    .height(PROPERTY_ROW)
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
    .height(PROPERTY_ROW)
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
    .height(PROPERTY_ROW + 4.0)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

/// Groupe de propriétés précédé d'un intitulé discret.
pub fn group<'a, Message: 'a>(
    title: &'a str,
    rows: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut body = column![].spacing(0);
    for entry in rows {
        body = body.push(entry);
    }
    column![typo::label(title), super::surface::divider(), body,]
        .spacing(space::XS)
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

/// Une propriété reste plus dense qu'une ligne de table, sans devenir illisible.
const _: () = assert!(PROPERTY_ROW < crate::ui::theme::metrics::size::ROW);
const _: () = assert!(PROPERTY_ROW >= 24.0);
