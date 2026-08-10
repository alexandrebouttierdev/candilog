//! Carte d'indicateur : libellé, valeur Geist Mono 28 px, couleur et filigrane teinté.

use super::typo;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::theme::metrics::{radius, space};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

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

/// Carte d'indicateur sur verre avec icône du ton posée en filigrane à droite.
pub fn metric_icon_tinted<'a, Message: 'a>(
    label: &'a str,
    value: String,
    tone: Tone,
    glyph: Icon,
) -> Element<'a, Message> {
    container(
        row![
            column![typo::caption(label), toned_value(value, tone)].spacing(space::XXS),
            Space::with_width(Length::Fill),
            icon::icon(glyph, 24.0, Ink::Toned(tone)),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .padding(space::XL)
    .width(Length::Fill)
    .style(styles::glass_card)
    .into()
}

/// Valeur mono 28 px semibold, colorée par un ton sémantique.
fn toned_value<'a>(value: String, tone: Tone) -> iced::widget::Text<'a> {
    typo::text_mono(value, font::METRIC, font::MONO_SEMIBOLD).style(styles::toned_text(tone))
}

#[cfg(test)]
mod tests {
    use super::{metric_icon_tinted, tinted_style};
    use crate::ui::components::icon::Icon;
    use crate::ui::theme::styles;
    use crate::ui::theme::tokens::NIGHT;
    use crate::ui::theme::{dark, Tone};
    use iced::{Background, Color, Element};

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

    #[test]
    fn la_carte_avec_icone_s_instancie_pour_chaque_ton_du_dashboard() {
        for tone in [Tone::Accent, Tone::Violet, Tone::Success, Tone::Warning] {
            let _: Element<'_, ()> =
                metric_icon_tinted("Libellé", "42".to_owned(), tone, Icon::Chart);
        }
    }

    #[test]
    fn la_valeur_porte_la_couleur_du_ton() {
        let style = styles::toned_text(Tone::Violet)(&dark());
        assert_eq!(style.color, Some(Tone::Violet.color(&NIGHT)));
    }
}
