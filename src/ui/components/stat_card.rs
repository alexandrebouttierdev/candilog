//! Carte d'indicateur : libellé, valeur Geist Mono 28 px, couleur et filigrane teinté.

use super::typo;
use crate::ui::theme::metrics::{radius, space};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{column, container};
use iced::{Background, Border, Color, Element, Length, Theme};

/// Carte d'indicateur simple : label, valeur mono, couleur.
pub fn metric<'a, Message: 'a>(
    label: &'a str,
    value: String,
    color: Color,
) -> Element<'a, Message> {
    container(
        column![
            typo::caption(label),
            typo::text_mono(value, font::METRIC, font::MONO_SEMIBOLD)
                .style(move |_theme| iced::widget::text::Style { color: Some(color) }),
        ]
        .spacing(space::XXS),
    )
    .padding(space::LG)
    .width(Length::Fill)
    .style(styles::glass_card)
    .into()
}

/// Style d'une carte teintée par un ton : fond `tone.surface` limité à 6 %, filet `tone.edge`.
pub fn tinted_style(tone: Tone) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let palette = tokens(theme);
        let mut surface = tone.surface(&palette);
        surface.a = 0.06;
        container::Style {
            background: Some(Background::Color(surface)),
            border: Border {
                color: tone.edge(&palette),
                width: 1.0,
                radius: radius::CARD.into(),
            },
            ..container::Style::default()
        }
    }
}

/// Carte d'indicateur en filigrane coloré (fond teinté par le ton).
pub fn metric_tinted<'a, Message: 'a>(
    label: &'a str,
    value: String,
    tone: Tone,
) -> Element<'a, Message> {
    container(
        column![
            typo::caption(label),
            typo::text_mono(value, font::METRIC, font::MONO_SEMIBOLD)
                .style(styles::toned_text(tone)),
        ]
        .spacing(space::XXS),
    )
    .padding(space::XL)
    .width(Length::Fill)
    .style(tinted_style(tone))
    .into()
}

#[cfg(test)]
mod tests {
    use super::tinted_style;
    use crate::ui::theme::tokens::NIGHT;
    use crate::ui::theme::{dark, Tone};
    use iced::{Background, Color};

    #[test]
    fn le_ton_teinte_le_fond_sans_ecraser_le_texte() {
        let surface = Tone::Violet.surface(&NIGHT);
        assert_eq!(surface.a, 0.14);
        let style = tinted_style(Tone::Violet)(&dark());
        assert_eq!(style.border.color, Tone::Violet.edge(&NIGHT));
    }

    #[test]
    fn le_fond_teinte_est_limite_a_six_pour_cent() {
        let style = tinted_style(Tone::Violet)(&dark());
        let Background::Color(color) = style.background.unwrap() else {
            panic!("fond absent")
        };
        assert!((color.a - 0.06).abs() < 1e-3);
        let surface = Tone::Violet.surface(&NIGHT);
        assert_eq!(
            color,
            Color {
                r: surface.r,
                g: surface.g,
                b: surface.b,
                a: 0.06,
            }
        );
    }
}
