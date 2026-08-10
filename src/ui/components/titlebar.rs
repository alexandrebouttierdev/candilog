//! Barre de titre : logo, nom de l'application, runtime et bascule de thème.

use super::icon::{self, Icon, Ink};
use super::typo;
use crate::ui::theme::metrics::{radius, size, space};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use iced::widget::{button, container, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Barre de titre de fenêtre (42 px).
pub fn titlebar<'a, Message: Clone + 'a>(
    runtime: Element<'a, Message>,
    is_dark: bool,
    on_toggle_theme: Message,
) -> Element<'a, Message> {
    row![
        row![
            container(icon::icon(Icon::Sparkles, 14.0, Ink::OnAccent))
                .width(22.0)
                .height(22.0)
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
            typo::body("Candilog").font(font::SEMIBOLD),
        ]
        .spacing(7.0)
        .align_y(Alignment::Center)
        .width(Length::Fixed(200.0)),
        Space::with_width(Length::Fill),
        runtime,
        Space::with_width(space::MD),
        button(icon::icon(
            if is_dark { Icon::Sun } else { Icon::Moon },
            14.0,
            Ink::Muted,
        ))
        .width(28.0)
        .height(28.0)
        .padding(0)
        .style(styles::ghost)
        .on_press(on_toggle_theme),
    ]
    .padding([0.0, space::LG])
    .height(size::TITLEBAR)
    .align_y(Alignment::Center)
    .into()
}

#[cfg(test)]
mod tests {
    use crate::ui::theme::metrics::size;

    #[test]
    fn la_barre_de_titre_fait_42_pixels() {
        assert_eq!(size::TITLEBAR, 42.0);
    }
}
