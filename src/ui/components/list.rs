//! Lignes de liste d'un volet maître ou d'un panneau latéral.
//!
//! Plus compactes qu'une table : un titre, une métadonnée, un accessoire.

use super::typo;
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use iced::widget::{button, column, container, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Ligne sélectionnable portant un appui, un titre et une métadonnée.
pub fn row_item<'a, Message: Clone + 'a>(
    leading: impl Into<Element<'a, Message>>,
    title: String,
    meta: String,
    trailing: impl Into<Element<'a, Message>>,
    selected: bool,
    on_press: Message,
) -> Element<'a, Message> {
    let body = row![
        marker(selected),
        leading.into(),
        column![typo::item(title), typo::caption(meta)].spacing(space::XXS),
        Space::with_width(Length::Fill),
        trailing.into(),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center);

    column![
        button(crate::ui::components::button::vcenter(body))
            .width(Length::Fill)
            .height(size::ROW_COMFORTABLE)
            .padding([0.0, space::LG])
            .style(styles::row_item(selected))
            .on_press(on_press),
        super::surface::divider(),
    ]
    .into()
}

/// Ligne compacte sur une seule ligne de texte.
pub fn row_compact<'a, Message: Clone + 'a>(
    leading: impl Into<Element<'a, Message>>,
    title: String,
    trailing: impl Into<Element<'a, Message>>,
    selected: bool,
    on_press: Message,
) -> Element<'a, Message> {
    let body = row![
        marker(selected),
        leading.into(),
        typo::item(title),
        Space::with_width(Length::Fill),
        trailing.into(),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center);

    column![
        button(crate::ui::components::button::vcenter(body))
            .width(Length::Fill)
            .height(size::ROW)
            .padding([0.0, space::LG])
            .style(styles::row_item(selected))
            .on_press(on_press),
        super::surface::divider(),
    ]
    .into()
}

/// Ligne informative non cliquable.
pub fn row_static<'a, Message: 'a>(
    leading: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    trailing: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        container(
            row![
                leading.into(),
                content.into(),
                Space::with_width(Length::Fill),
                trailing.into(),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
        )
        .height(size::ROW)
        .padding([0.0, space::LG])
        .width(Length::Fill)
        .align_y(Alignment::Center),
        super::surface::divider(),
    ]
    .into()
}

fn marker<'a, Message: 'a>(selected: bool) -> Element<'a, Message> {
    container(Space::new(stroke::MARKER, size::LIST_MARKER))
        .style(move |theme: &Theme| container::Style {
            background: selected.then(|| Background::Color(tokens(theme).accent)),
            border: Border {
                radius: radius::MARKER.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
}
