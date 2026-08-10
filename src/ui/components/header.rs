//! En-têtes de page et de modale.

use super::button as controls;
use super::icon::{self, Icon, Ink};
use super::typo;
use crate::ui::theme::metrics::{radius, space};
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use iced::widget::{column, container, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

/// Rayon de l'en-tête de page (rounded-2xl).
pub const PAGE_HEADER_RADIUS: f32 = 16.0;

/// En-tête de page : icône, titre, description, actions.
pub fn page_header<'a, Message: 'a>(
    glyph: Icon,
    title: &'a str,
    description: &'a str,
    actions: Element<'a, Message>,
) -> Element<'a, Message> {
    container(
        row![
            container(icon::icon(glyph, icon::LG, Ink::OnAccent))
                .width(44.0)
                .height(44.0)
                .style(move |theme: &Theme| {
                    let palette = tokens(theme);
                    container::Style {
                        background: Some(Background::Color(palette.accent_fill)),
                        border: Border {
                            radius: radius::CONTROL.into(),
                            ..Border::default()
                        },
                        ..container::Style::default()
                    }
                }),
            column![typo::title(title), typo::meta(description),].spacing(space::XS),
            iced::widget::Space::with_width(Length::Fill),
            actions,
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .padding([space::XL, space::XXL])
    .width(Length::Fill)
    .style(move |theme: &Theme| {
        let palette = tokens(theme);
        container::Style {
            background: Some(Background::Color(palette.panel)),
            border: Border {
                color: Color {
                    a: 0.60,
                    ..palette.border
                },
                width: 1.0,
                radius: PAGE_HEADER_RADIUS.into(),
            },
            ..container::Style::default()
        }
    })
    .into()
}

/// En-tête de modale : icône, titre, description, bouton fermer.
pub fn form_modal_header<'a, Message: Clone + 'a>(
    glyph: Icon,
    title: &'a str,
    description: &'a str,
    on_close: Message,
) -> Element<'a, Message> {
    row![
        container(icon::icon(glyph, icon::MD, Ink::OnAccent))
            .width(40.0)
            .height(40.0)
            .style(move |theme: &Theme| {
                let palette = tokens(theme);
                container::Style {
                    background: Some(Background::Color(palette.accent_fill)),
                    border: Border {
                        radius: radius::CONTROL.into(),
                        ..Border::default()
                    },
                    ..container::Style::default()
                }
            }),
        column![
            text(title).size(16.0).font(font::SEMIBOLD),
            typo::caption(description),
        ]
        .spacing(space::XXS),
        iced::widget::Space::with_width(Length::Fill),
        controls::icon_action(Icon::Close, "Fermer", on_close),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center)
    .into()
}

#[cfg(test)]
mod tests {
    use super::PAGE_HEADER_RADIUS;

    #[test]
    fn le_rayon_du_header_de_page_suit_le_handoff() {
        assert_eq!(PAGE_HEADER_RADIUS, 16.0);
    }
}
