//! Barre d'état supérieure : runtime IA et respiration du chrome natif.

use crate::ui::theme::metrics::{size, space};
use crate::ui::theme::tokens::tokens;
use iced::widget::{container, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Barre supérieure compacte. La marque et le thème appartiennent désormais au rail.
pub fn titlebar<'a, Message: 'a>(runtime: Element<'a, Message>) -> Element<'a, Message> {
    container(
        row![Space::with_width(Length::Fill), runtime]
            .align_y(Alignment::Center)
            .padding([0.0, space::LG]),
    )
    .width(Length::Fill)
    .height(size::TITLEBAR)
    .align_y(Alignment::Center)
    .style(|theme: &Theme| {
        let palette = tokens(theme);
        container::Style {
            background: Some(Background::Color(palette.chrome)),
            border: Border {
                color: palette.border,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        }
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::titlebar;
    use crate::ui::theme::metrics::size;

    #[test]
    fn la_barre_de_titre_fait_36_pixels() {
        assert_eq!(size::TITLEBAR, 36.0);
    }

    #[test]
    fn la_barre_de_titre_s_instancie() {
        let _: iced::Element<'_, ()> = titlebar(iced::widget::Space::with_width(0).into());
    }
}
