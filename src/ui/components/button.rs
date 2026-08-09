//! Hiérarchie de boutons de Candilog.
//!
//! Règle : au plus une action `primary` visible par écran. Les contrôles de
//! toolbar et les actions de ligne sont plus compacts que les actions de
//! dialogue.

use super::icon::{self, Icon, Ink};
use super::typo;
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use iced::widget::{container, row, text, tooltip, Button};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Compose une icône et un libellé dans l'ordre et l'espacement canoniques.
fn face<'a, Message: 'a>(kind: Option<Icon>, ink: Ink, label: &'a str) -> Element<'a, Message> {
    let text_part = text(label).size(font::BODY);
    match kind {
        Some(kind) => row![icon::icon(kind, icon::SM, ink), text_part]
            .spacing(space::SM)
            .align_y(Alignment::Center)
            .into(),
        None => text_part.into(),
    }
}

/// Action principale de l'écran, avec icône optionnelle.
pub fn primary<'a, Message: 'a>(label: &'a str, kind: Option<Icon>) -> Button<'a, Message> {
    iced::widget::button(face(kind, Ink::OnAccent, label))
        .height(size::CONTROL)
        .padding([0.0, 10.0])
        .style(styles::primary)
}

/// Action secondaire d'un dialogue ou d'une section.
pub fn secondary<'a, Message: 'a>(label: &'a str, kind: Option<Icon>) -> Button<'a, Message> {
    iced::widget::button(face(kind, Ink::Muted, label))
        .height(size::CONTROL)
        .padding([0.0, 10.0])
        .style(styles::secondary)
}

/// Contrôle de toolbar ou action de section, sans surface au repos.
pub fn ghost<'a, Message: 'a>(label: &'a str, kind: Option<Icon>) -> Button<'a, Message> {
    iced::widget::button(face(kind, Ink::Muted, label))
        .height(size::CONTROL)
        .padding([0.0, 8.0])
        .style(styles::ghost)
}

/// Action destructive discrète.
pub fn danger<'a, Message: 'a>(label: &'a str, kind: Option<Icon>) -> Button<'a, Message> {
    iced::widget::button(face(
        kind,
        Ink::Toned(crate::ui::theme::Tone::Danger),
        label,
    ))
    .height(size::CONTROL)
    .padding([0.0, 10.0])
    .style(styles::danger)
}

/// Confirmation destructive d'un dialogue.
pub fn danger_filled<'a, Message: 'a>(label: &'a str) -> Button<'a, Message> {
    iced::widget::button(text(label).size(font::BODY))
        .height(size::CONTROL)
        .padding([0.0, 10.0])
        .style(styles::danger_filled)
}

/// Élément d'un contrôle segmenté.
pub fn segment<'a, Message: 'a>(label: impl Into<String>, active: bool) -> Button<'a, Message> {
    iced::widget::button(text(label.into()).size(font::BODY).font(if active {
        font::MEDIUM
    } else {
        font::REGULAR
    }))
    .height(size::CONTROL - 4.0)
    .padding([0.0, 10.0])
    .style(if active {
        styles::selected
    } else {
        styles::ghost
    })
}

/// Regroupe des segments dans un cadre unique.
pub fn segmented<'a, Message: Clone + 'a>(
    segments: impl IntoIterator<Item = Button<'a, Message>>,
) -> Element<'a, Message> {
    let mut group = row![].spacing(2);
    for segment in segments {
        group = group.push(segment);
    }
    container(group)
        .padding(2)
        .style(|theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(palette.sunken)),
                border: Border {
                    color: palette.border,
                    width: stroke::HAIRLINE,
                    radius: radius::CONTROL.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Bouton purement iconique, toujours accompagné d'une aide au survol.
pub fn icon_action<'a, Message: Clone + 'a>(
    kind: Icon,
    hint: &'a str,
    on_press: Message,
) -> Element<'a, Message> {
    with_hint(
        iced::widget::button(icon::icon(kind, icon::SM, Ink::Muted))
            .width(size::ICON_BUTTON)
            .height(size::ICON_BUTTON)
            .padding(0)
            .style(styles::ghost)
            .on_press(on_press),
        hint,
    )
}

/// Bouton iconique destructif.
pub fn icon_danger<'a, Message: Clone + 'a>(
    kind: Icon,
    hint: &'a str,
    on_press: Message,
) -> Element<'a, Message> {
    with_hint(
        iced::widget::button(icon::icon(kind, icon::SM, Ink::Muted))
            .width(size::ICON_BUTTON)
            .height(size::ICON_BUTTON)
            .padding(0)
            .style(styles::danger)
            .on_press(on_press),
        hint,
    )
}

/// Enveloppe un contrôle d'une infobulle desktop.
pub fn with_hint<'a, Message: 'a>(
    control: impl Into<Element<'a, Message>>,
    hint: &'a str,
) -> Element<'a, Message> {
    tooltip(control, typo::caption(hint), tooltip::Position::Bottom)
        .gap(4)
        .padding(6)
        .style(|theme: &Theme| container::Style {
            border: Border {
                radius: radius::CONTROL.into(),
                ..styles::raised(theme).border
            },
            ..styles::raised(theme)
        })
        .into()
}

/// Bouton occupant toute la largeur de son conteneur, pour un volet latéral.
pub fn wide<'a, Message: 'a>(label: &'a str, kind: Option<Icon>) -> Button<'a, Message> {
    secondary(label, kind).width(Length::Fill)
}
