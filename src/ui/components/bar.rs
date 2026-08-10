//! Barre de proportion : libellé, valeur, piste et remplissage coloré.

use super::typo;
use crate::ui::theme::metrics::{radius, space};
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Borne la proportion affichée entre 0 et 100.
#[must_use]
pub fn borne(ratio: f32) -> f32 {
    ratio.clamp(0.0, 100.0)
}

/// Décompose une proportion en portions de piste : remplissage coloré et
/// reste vide, sur une échelle de 10 000 unités. Le total vaut toujours
/// 10 000, même hors bornes (0 ou 100 %).
#[must_use]
pub fn proportions(ratio: f32) -> (u16, u16) {
    let fill = ((ratio * 100.0).round() as u16).clamp(0, 10_000);
    (fill, 10_000_u16.saturating_sub(fill))
}

/// Barre de proportion verticalement compacte.
pub fn barre<'a, Message: 'a>(
    label: &'a str,
    valeur: impl Into<String>,
    ratio: f32,
    tone: Tone,
) -> Element<'a, Message> {
    let (fill_portion, rest) = proportions(ratio);

    let fill = if fill_portion > 0 {
        Some(
            container(Space::new(Length::Fill, Length::Fixed(12.0)))
                .width(Length::FillPortion(fill_portion))
                .height(Length::Fixed(12.0))
                .style(move |theme: &Theme| {
                    let palette = tokens(theme);
                    container::Style {
                        background: Some(Background::Color(tone.color(&palette))),
                        border: Border {
                            radius: radius::PILL.into(),
                            ..Border::default()
                        },
                        ..container::Style::default()
                    }
                }),
        )
    } else {
        None
    };
    let rest_child = if rest > 0 {
        Some(
            container(Space::new(Length::Fill, Length::Fixed(12.0)))
                .width(Length::FillPortion(rest))
                .height(Length::Fixed(12.0)),
        )
    } else {
        None
    };

    column![
        row![
            typo::body(label),
            Space::with_width(Length::Fill),
            typo::caption(valeur.into()),
        ]
        .align_y(Alignment::Center),
        container(
            iced::widget::Row::new()
                .push_maybe(fill)
                .push_maybe(rest_child)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .style(move |theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(palette.sunken)),
                border: Border {
                    radius: radius::PILL.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }
        }),
    ]
    .spacing(space::XS)
    .into()
}

#[cfg(test)]
mod tests {
    use super::{borne, proportions};

    #[test]
    fn la_proportion_est_bornee_entre_zero_et_cent() {
        assert_eq!(borne(-5.0), 0.0);
        assert_eq!(borne(0.0), 0.0);
        assert_eq!(borne(50.0), 50.0);
        assert_eq!(borne(150.0), 100.0);
        assert_eq!(proportions(-5.0), (0, 10_000));
        assert_eq!(proportions(150.0), (10_000, 0));
    }

    #[test]
    fn la_piste_conserve_une_part_vide_sous_la_barre() {
        let (fill, rest) = proportions(30.0);
        assert_eq!(fill, 3_000);
        assert_eq!(rest, 7_000);
        assert_eq!(fill + rest, 10_000);
    }
}
