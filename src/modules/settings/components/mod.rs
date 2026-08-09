//! Rendu des lignes de réglage.

use crate::ui::components::{layout, surface, typo};
use crate::ui::theme::metrics::space;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Ligne de réglage : intitulé, description et contrôle aligné à droite.
pub fn setting<'a, Message: 'a>(
    title: &'a str,
    description: &'a str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        container(
            row![
                column![typo::body(title), typo::caption(description)].spacing(1),
                layout::spacer(),
                control.into(),
            ]
            .spacing(space::XXL)
            .align_y(Alignment::Center),
        )
        .padding([space::LG, 0.0])
        .width(Length::Fill),
        surface::divider(),
    ]
    .into()
}

/// Ligne de réglage dont le contrôle occupe toute la largeur sous l'intitulé.
pub fn setting_stacked<'a, Message: 'a>(
    title: &'a str,
    description: &'a str,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        container(
            column![
                column![typo::body(title), typo::caption(description)].spacing(1),
                control.into(),
            ]
            .spacing(space::MD),
        )
        .padding([space::LG, 0.0])
        .width(Length::Fill),
        surface::divider(),
    ]
    .into()
}

/// Groupe d'actions d'une section de réglages.
pub fn actions<'a, Message: 'a>(
    title: &'a str,
    description: &'a str,
    buttons: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut line = row![].spacing(space::SM).align_y(Alignment::Center);
    for control in buttons {
        line = line.push(control);
    }
    setting(title, description, line)
}
