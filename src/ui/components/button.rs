//! Hiérarchie de boutons de Candilog.
//!
//! Règle : au plus une action `primary` visible par écran. Les contrôles de
//! toolbar et les actions de ligne sont plus compacts que les actions de
//! dialogue.

use super::icon::{self, Icon, Ink};
use super::tooltip;
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use iced::widget::{container, row, text, Button};
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
        .height(size::ACTION)
        .padding([0.0, 13.0])
        .style(styles::primary)
}

/// Action secondaire d'un dialogue ou d'une section.
pub fn secondary<'a, Message: 'a>(label: &'a str, kind: Option<Icon>) -> Button<'a, Message> {
    iced::widget::button(face(kind, Ink::Muted, label))
        .height(size::ACTION)
        .padding([0.0, 13.0])
        .style(styles::secondary)
}

/// Contrôle de toolbar ou action de section, sans surface au repos.
pub fn ghost<'a, Message: 'a>(label: &'a str, kind: Option<Icon>) -> Button<'a, Message> {
    iced::widget::button(face(kind, Ink::Muted, label))
        .height(size::ACTION)
        .padding([0.0, space::MD])
        .style(styles::ghost)
}

/// Action destructive discrète.
pub fn danger<'a, Message: 'a>(label: &'a str, kind: Option<Icon>) -> Button<'a, Message> {
    iced::widget::button(face(
        kind,
        Ink::Toned(crate::ui::theme::Tone::Danger),
        label,
    ))
    .height(size::ACTION)
    .padding([0.0, 13.0])
    .style(styles::danger)
}

/// Confirmation destructive d'un dialogue.
pub fn danger_filled<'a, Message: 'a>(label: &'a str) -> Button<'a, Message> {
    iced::widget::button(text(label).size(font::BODY))
        .height(size::ACTION)
        .padding([0.0, 13.0])
        .style(styles::danger_filled)
}

/// Élément d'un contrôle segmenté.
///
/// Plus bas que `size::CONTROL` de deux fois `space::XXS`, exactement
/// l'espace pris par le padding du cadre `segmented` qui l'enveloppe : la
/// somme des deux retombe sur `size::CONTROL`.
pub fn segment<'a, Message: 'a>(label: impl Into<String>, active: bool) -> Button<'a, Message> {
    iced::widget::button(text(label.into()).size(font::BODY).font(if active {
        font::MEDIUM
    } else {
        font::REGULAR
    }))
    .height(size::CONTROL - 2.0 * space::XXS)
    .padding([0.0, space::SM + space::XXS])
    .style(if active {
        styles::selected_inverse
    } else {
        styles::ghost
    })
}

/// Regroupe des segments dans un cadre unique.
pub fn segmented<'a, Message: Clone + 'a>(
    segments: impl IntoIterator<Item = Button<'a, Message>>,
) -> Element<'a, Message> {
    let mut group = row![].spacing(space::XXS);
    for segment in segments {
        group = group.push(segment);
    }
    container(group)
        .padding(space::XXS)
        .style(|theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(alpha(palette.sunken, 0.7))),
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

/// Bouton iconique d'action principale.
pub fn icon_primary<'a, Message: Clone + 'a>(
    kind: Icon,
    hint: &'a str,
    on_press: Message,
) -> Element<'a, Message> {
    with_hint(
        iced::widget::button(icon::icon(kind, icon::SM, Ink::OnAccent))
            .width(size::ICON_BUTTON)
            .height(size::ICON_BUTTON)
            .padding(0)
            .style(styles::primary)
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
///
/// Délègue au composant `tooltip` partagé : c'est la seule façon de dessiner
/// une infobulle dans l'application. Toujours positionnée en dessous, comme
/// il convient à un bouton de toolbar.
pub fn with_hint<'a, Message: 'a>(
    control: impl Into<Element<'a, Message>>,
    hint: &'a str,
) -> Element<'a, Message> {
    tooltip::tip(control, hint, tooltip::Side::Bottom)
}

/// Bouton occupant toute la largeur de son conteneur, pour un volet latéral.
pub fn wide<'a, Message: 'a>(label: &'a str, kind: Option<Icon>) -> Button<'a, Message> {
    secondary(label, kind).width(Length::Fill)
}
