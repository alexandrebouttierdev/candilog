//! Carte d'indicateur : libellé, valeur Geist Mono 28 px, couleur et filigrane teinté.

use super::typo;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Color, Element, Length};

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

/// Indicateur desktop compact : puits d'icône à gauche, valeur et libellé.
pub fn metric_icon_tinted<'a, Message: 'a>(
    label: &'a str,
    value: String,
    tone: Tone,
    glyph: Icon,
) -> Element<'a, Message> {
    let mut value = typo::text_mono(value, font::METRIC, font::MONO_SEMIBOLD);
    if tone != Tone::Accent {
        value = value.style(styles::toned_text(tone));
    }

    container(
        row![
            container(icon::icon(glyph, icon::MD, Ink::Toned(tone)))
                .width(36.0)
                .height(36.0)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .style(styles::icon_tile(tone)),
            column![typo::caption(label), value].spacing(space::XXS),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .height(84.0)
    .padding(space::LG)
    .width(Length::Fill)
    .style(styles::glass_card)
    .into()
}

#[cfg(test)]
mod tests {
    use super::metric_icon_tinted;
    use crate::ui::components::icon::Icon;
    use crate::ui::theme::styles;
    use crate::ui::theme::tokens::NIGHT;
    use crate::ui::theme::{dark, Tone};
    use iced::Element;

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
