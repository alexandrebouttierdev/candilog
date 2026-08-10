//! Badge de score coloré par tranche (handoff candilog-desktop `scoreColor`).

use super::typo;
use crate::ui::theme::metrics::radius;
use crate::ui::theme::typography as font;
use iced::widget::{column, container};
use iced::{Background, Border, Color, Element, Length};

/// Convertit une teinte `hsl(h s% l%)` (h en degrés, s et l en %) en couleur opaque.
///
/// Recette sRGB standard, copie privée de `theme::tokens` : les couleurs des
/// tranches sont fixées par le handoff, pas résolues depuis le thème actif.
const fn hsl(h: f32, s: f32, l: f32) -> Color {
    let s = (s / 100.0).clamp(0.0, 1.0);
    let l = (l / 100.0).clamp(0.0, 1.0);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = (h % 360.0 + 360.0) % 360.0 / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let (r1, g1, b1) = match h_prime as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    Color {
        r: r1 + m,
        g: g1 + m,
        b: b1 + m,
        a: 1.0,
    }
}

/// Tranche du score : indigo ≥85, vert ≥70, ambre ≥50, rouge sinon.
#[must_use]
pub fn score_tier(score: u8) -> &'static str {
    if score >= 85 {
        "indigo"
    } else if score >= 70 {
        "vert"
    } else if score >= 50 {
        "ambre"
    } else {
        "rouge"
    }
}

/// Couleurs (fond, texte) de la tranche donnée.
#[must_use]
pub fn score_colors(tier: &str) -> (Color, Color) {
    match tier {
        "indigo" => (hsl(226.0, 100.0, 95.0), hsl(245.0, 58.0, 44.0)),
        "vert" => (hsl(142.0, 71.0, 90.0), hsl(142.0, 60.0, 30.0)),
        "ambre" => (hsl(38.0, 92.0, 90.0), hsl(38.0, 80.0, 35.0)),
        _ => (hsl(0.0, 84.0, 90.0), hsl(0.0, 70.0, 40.0)),
    }
}

/// Badge de score : carte 80×80 colorée par tranche, chiffre mono 24 bold.
pub fn score_badge<'a, Message: 'a>(score: u8) -> Element<'a, Message> {
    let (background, foreground) = score_colors(score_tier(score));
    container(
        column![
            typo::text_mono(score.to_string(), 24.0, font::MONO_SEMIBOLD).style(move |_theme| {
                iced::widget::text::Style {
                    color: Some(foreground),
                }
            }),
            typo::caption("ATS"),
        ]
        .spacing(0)
        .align_x(iced::Alignment::Center),
    )
    .width(80.0)
    .height(80.0)
    .center_x(Length::Fixed(80.0))
    .center_y(Length::Fixed(80.0))
    .style(move |_theme| container::Style {
        background: Some(Background::Color(background)),
        border: Border {
            radius: radius::PANEL.into(),
            ..Border::default()
        },
        ..container::Style::default()
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::{score_colors, score_tier};

    #[test]
    fn les_tranches_de_score_suivent_le_handoff() {
        assert_eq!(score_tier(95), "indigo");
        assert_eq!(score_tier(85), "indigo");
        assert_eq!(score_tier(70), "vert");
        assert_eq!(score_tier(50), "ambre");
        assert_eq!(score_tier(10), "rouge");
    }

    #[test]
    fn les_couleurs_de_chaque_tranche_sont_distinctes() {
        let pairs = [
            score_colors("indigo"),
            score_colors("vert"),
            score_colors("ambre"),
            score_colors("rouge"),
        ];
        for (index, (fond, texte)) in pairs.iter().enumerate() {
            assert_ne!(fond, texte, "fond et texte d'une tranche identiques");
            for (autre_fond, _) in &pairs[index + 1..] {
                assert_ne!(fond, autre_fond, "deux tranches partagent le même fond");
            }
        }
        assert_eq!(score_colors("inconnue"), score_colors("rouge"));
    }
}
