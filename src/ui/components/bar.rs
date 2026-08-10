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

/// Barre de proportion verticalement compacte.
pub fn barre<'a, Message: 'a>(
    label: &'a str,
    valeur: impl Into<String>,
    ratio: f32,
    tone: Tone,
) -> Element<'a, Message> {
    let proportion = (borne(ratio) * 100.0) as u16;
    let fill: Element<'a, Message> = if proportion > 0 {
        container(Space::new(
            Length::FillPortion(proportion),
            Length::Fixed(12.0),
        ))
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
        })
        .into()
    } else {
        Space::with_height(12.0).into()
    };

    column![
        row![
            typo::body(label),
            Space::with_width(Length::Fill),
            typo::caption(valeur.into()),
        ]
        .align_y(Alignment::Center),
        container(fill)
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
    use super::borne;

    #[test]
    fn la_proportion_est_bornee_entre_zero_et_cent() {
        assert_eq!(borne(-5.0), 0.0);
        assert_eq!(borne(0.0), 0.0);
        assert_eq!(borne(50.0), 50.0);
        assert_eq!(borne(150.0), 100.0);
    }
}
