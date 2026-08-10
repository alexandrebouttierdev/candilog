//! Barre d'outils d'écran.
//!
//! Une seule toolbar par écran, hauteur `size::TOOLBAR`, jamais de titre
//! « hero » ni de sous-titre marketing. Composition : titre, compteur, puis
//! groupes de contrôles séparés par une gouttière `space::XL`, actions à
//! droite.

use super::button;
use super::icon::Icon;
use super::typo;
use crate::ui::theme::metrics::{size, space, stroke};
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::Layout;
use iced::widget::{container, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Assemble la barre d'outils d'un écran.
pub fn toolbar<'a, Message: 'a>(
    title: &'a str,
    leading: impl Into<Element<'a, Message>>,
    trailing: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        row![
            typo::title(title),
            leading.into(),
            Space::with_width(Length::Fill),
            trailing.into(),
        ]
        .spacing(space::XL)
        .align_y(Alignment::Center),
    )
    .height(size::TOOLBAR)
    .padding([0.0, space::XL])
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let palette = tokens(theme);
        container::Style {
            background: Some(Background::Color(palette.chrome)),
            text_color: Some(palette.text),
            border: Border {
                color: palette.border,
                width: stroke::HAIRLINE,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        }
    })
    .into()
}

/// Regroupe des contrôles liés au sein d'une toolbar.
pub fn group<'a, Message: 'a>(
    controls: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut line = row![].spacing(space::SM).align_y(Alignment::Center);
    for control in controls {
        line = line.push(control);
    }
    line.into()
}

/// Action de toolbar. Sous [`Layout::toolbar_action_labels`], une action
/// iconique se replie sur son icône seule, l'intitulé complet restitué en
/// infobulle au survol — le motif déjà employé par le rail replié. Une
/// action sans icône ne peut pas se replier sans devenir indevinable : elle
/// garde toujours son libellé.
pub fn action<'a, Message: Clone + 'a>(
    layout: Layout,
    label: &'a str,
    icon: Option<Icon>,
    on_press: Message,
) -> Element<'a, Message> {
    match icon {
        Some(kind) if !layout.toolbar_action_labels() => button::icon_action(kind, label, on_press),
        _ => button::ghost(label, icon).on_press(on_press).into(),
    }
}

/// Séparateur vertical entre deux groupes de toolbar.
pub fn separator<'a, Message: 'a>() -> Element<'a, Message> {
    container(Space::new(stroke::HAIRLINE, size::TOOLBAR_SEPARATOR))
        .style(|theme: &Theme| container::Style {
            background: Some(Background::Color(tokens(theme).border_strong)),
            ..container::Style::default()
        })
        .into()
}

/// Bande secondaire sous la toolbar : jetons de filtres, sélection, contexte.
pub fn strip<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .height(size::TOOLBAR_STRIP)
        .padding([0.0, space::XL])
        .width(Length::Fill)
        .align_y(Alignment::Center)
        .style(|theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(palette.sunken)),
                text_color: Some(palette.text),
                border: Border {
                    color: palette.border,
                    width: stroke::HAIRLINE,
                    radius: 0.0.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Barre d'état de bas de fenêtre.
pub fn status_bar<'a, Message: 'a>(
    left: impl Into<Element<'a, Message>>,
    right: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        row![left.into(), Space::with_width(Length::Fill), right.into(),]
            .spacing(space::XL)
            .align_y(Alignment::Center),
    )
    .height(size::STATUS_BAR)
    .padding([0.0, space::XL])
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let palette = tokens(theme);
        container::Style {
            background: Some(Background::Color(palette.chrome)),
            text_color: Some(palette.text_secondary),
            border: Border {
                color: palette.border,
                width: stroke::HAIRLINE,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        }
    })
    .into()
}
