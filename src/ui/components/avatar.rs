//! Avatar à initiales : carré arrondi, fond teinté, initiales en graisse forte.

use super::typo;
use crate::ui::theme::metrics::radius;
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::container;
use iced::widget::text::IntoFragment;
use iced::{Background, Border, Element, Length, Theme};

/// Rayon du fond selon la taille de l'avatar : contrôle jusqu'à 38 px,
/// panneau au-delà.
#[must_use]
pub const fn radius_for(size: f32) -> f32 {
    if size <= 38.0 {
        radius::CONTROL
    } else {
        radius::PANEL
    }
}

/// Initiales d'un nom : première lettre du premier et du dernier mot, en
/// majuscules ; « ? » quand le nom est vide.
#[must_use]
pub fn initials_of(name: &str) -> String {
    let mut words = name.split_whitespace();
    let Some(first) = words.next() else {
        return "?".into();
    };
    let Some(first_initial) = first.chars().next() else {
        return "?".into();
    };
    let mut initials = first_initial.to_uppercase().collect::<String>();
    if let Some(initial) = words.last().and_then(|word| word.chars().next()) {
        initials.push_str(&initial.to_uppercase().collect::<String>());
    }
    initials
}

/// Avatar à initiales de la taille donnée, sur fond teinté par un ton.
pub fn avatar<'a, Message: 'a>(
    initials: impl IntoFragment<'a>,
    size: f32,
    tint: Tone,
) -> Element<'a, Message> {
    container(
        typo::text_mono(initials, size * 0.34, font::MONO_SEMIBOLD).style(styles::toned_text(tint)),
    )
    .width(size)
    .height(size)
    .center(Length::Fixed(size))
    .style(move |theme: &Theme| {
        let palette = tokens(theme);
        container::Style {
            background: Some(Background::Color(tint.surface(&palette))),
            border: Border {
                radius: radius_for(size).into(),
                ..Border::default()
            },
            ..container::Style::default()
        }
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::{avatar, initials_of, radius_for};
    use crate::ui::theme::metrics::radius;
    use crate::ui::theme::Tone;
    use iced::Element;

    #[test]
    fn les_initiales_prennent_le_premier_et_le_dernier_mot() {
        assert_eq!(initials_of("Alice Dupont"), "AD");
        assert_eq!(initials_of("alice dupont"), "AD");
        assert_eq!(initials_of("Alice Marie Dupont"), "AD");
    }

    #[test]
    fn un_nom_d_un_seul_mot_ne_donne_qu_une_initiale() {
        assert_eq!(initials_of("Alice"), "A");
        assert_eq!(initials_of("A"), "A");
    }

    #[test]
    fn sans_nom_l_avatar_porte_un_point_d_interrogation() {
        assert_eq!(initials_of(""), "?");
        assert_eq!(initials_of("   "), "?");
    }

    #[test]
    fn le_rayon_suit_la_taille_de_l_avatar() {
        assert_eq!(radius_for(36.0), radius::CONTROL);
        assert_eq!(radius_for(38.0), radius::CONTROL);
        assert_eq!(radius_for(40.0), radius::PANEL);
        assert_eq!(radius_for(48.0), radius::PANEL);
    }

    #[test]
    fn l_avatar_s_instancie_aux_deux_tailles_de_l_ecran() {
        let _: Element<'_, ()> = avatar("AD", 36.0, Tone::Accent);
        let _: Element<'_, ()> = avatar("AD", 48.0, Tone::Neutral);
    }
}
