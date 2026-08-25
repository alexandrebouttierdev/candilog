//! En-têtes de page et de modale.

use super::button as controls;
use super::icon::{self, Icon, Ink};
use super::typo;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::text::IntoFragment;
use iced::widget::{column, container, row, text, vertical_rule};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// La toolbar touche les bords de la zone de contenu.
pub const PAGE_HEADER_RADIUS: f32 = 0.0;

/// En-tête de page : icône, titre, description, actions.
pub fn page_header<'a, Message: 'a>(
    glyph: Icon,
    title: &'a str,
    description: impl IntoFragment<'a>,
    actions: Element<'a, Message>,
) -> Element<'a, Message> {
    container(
        row![
            page_glyph(glyph),
            typo::title(title),
            container(vertical_rule(1).style(styles::divider)).height(18.0),
            typo::meta(description),
            iced::widget::Space::with_width(Length::Fill),
            actions,
        ]
        .spacing(space::LG)
        .align_y(Alignment::Center),
    )
    .height(58.0)
    .padding([0.0, space::XXL])
    .width(Length::Fill)
    .align_y(Alignment::Center)
    .style(move |theme: &Theme| {
        let palette = tokens(theme);
        container::Style {
            background: Some(Background::Color(palette.canvas)),
            border: Border {
                color: palette.border,
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
        container(icon::icon(glyph, icon::MD, Ink::Accent))
            .width(40.0)
            .height(40.0)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .style(styles::icon_tile(Tone::Accent)),
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

fn page_glyph<'a, Message: 'a>(glyph: Icon) -> Element<'a, Message> {
    container(icon::icon(glyph, icon::SM, Ink::Accent))
        .width(28.0)
        .height(28.0)
        .center(Length::Fixed(28.0))
        .style(styles::icon_tile(Tone::Accent))
        .into()
}

#[cfg(test)]
mod tests {
    use super::PAGE_HEADER_RADIUS;

    #[test]
    fn le_header_de_page_est_une_toolbar_sans_carte_arrondie() {
        assert_eq!(PAGE_HEADER_RADIUS, 0.0);
    }
}
