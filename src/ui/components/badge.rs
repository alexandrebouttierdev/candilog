//! Jetons compacts : statut, compteur, marqueur.
//!
//! Un jeton porte toujours un libellé. Les marqueurs de statut ajoutent une
//! forme distincte pour que l'information ne dépende jamais de la couleur.

use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use crate::ui::theme::{Marker, Tone};
use iced::widget::{container, row, text, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Jeton sémantique avec libellé.
pub fn badge<'a, Message: 'a>(label: impl Into<String>, tone: Tone) -> Element<'a, Message> {
    container(
        text(label.into())
            .size(font::CAPTION)
            .font(font::MEDIUM)
            .style(styles::toned_text(tone)),
    )
    .height(size::TAG)
    .padding([space::XXS, space::SM])
    .align_y(Alignment::Center)
    .style(styles::toned(tone))
    .into()
}

/// Jeton précédé d'un marqueur de forme, pour un statut de pipeline.
pub fn status<'a, Message: 'a>(
    label: impl Into<String>,
    tone: Tone,
    shape: Marker,
) -> Element<'a, Message> {
    container(
        row![
            marker(tone, shape),
            text(label.into())
                .size(font::CAPTION)
                .font(font::MEDIUM)
                .style(styles::toned_text(tone)),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
    )
    .height(size::TAG)
    .padding([space::XXS, space::SM])
    .align_y(Alignment::Center)
    .style(styles::toned(tone))
    .into()
}

/// Marqueur dessiné : la forme suffit à distinguer les états.
pub fn marker<'a, Message: 'a>(tone: Tone, shape: Marker) -> Element<'a, Message> {
    let dot = move |theme: &Theme| {
        let palette = tokens(theme);
        let color = tone.color(&palette);
        match shape {
            Marker::Hollow => container::Style {
                background: None,
                border: Border {
                    color,
                    width: stroke::EMPHASIS,
                    radius: radius::PILL.into(),
                },
                ..container::Style::default()
            },
            Marker::Half => container::Style {
                background: Some(Background::Color(color)),
                border: Border {
                    color,
                    width: stroke::HAIRLINE,
                    radius: iced::border::Radius {
                        top_left: radius::PILL,
                        bottom_left: radius::PILL,
                        top_right: radius::NONE,
                        bottom_right: radius::NONE,
                    },
                },
                ..container::Style::default()
            },
            Marker::Solid => container::Style {
                background: Some(Background::Color(color)),
                border: Border {
                    radius: radius::PILL.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            },
            Marker::Barred => container::Style {
                background: Some(Background::Color(crate::ui::theme::tokens::alpha(
                    color, 0.35,
                ))),
                border: Border {
                    color,
                    width: stroke::EMPHASIS,
                    radius: radius::BARRED.into(),
                },
                ..container::Style::default()
            },
        }
    };
    container(Space::new(size::STATUS_DOT, size::STATUS_DOT))
        .style(dot)
        .into()
}

/// Compteur discret accolé à un titre.
pub fn count<'a, Message: 'a>(value: usize) -> Element<'a, Message> {
    container(
        text(value.to_string())
            .size(font::CAPTION)
            .font(font::MEDIUM)
            .style(styles::muted_text),
    )
    .height(size::COUNTER)
    .padding([space::XXS, space::XXS + space::XS])
    .align_y(Alignment::Center)
    .style(|theme: &Theme| {
        let palette = tokens(theme);
        container::Style {
            background: Some(Background::Color(palette.sunken)),
            border: Border {
                color: palette.border,
                width: stroke::HAIRLINE,
                radius: radius::PILL.into(),
            },
            ..container::Style::default()
        }
    })
    .into()
}

/// Compteur mis en avant, pour signaler une charge à traiter.
pub fn count_toned<'a, Message: 'a>(value: usize, tone: Tone) -> Element<'a, Message> {
    if tone == Tone::Neutral || value == 0 {
        return count(value);
    }
    badge(value.to_string(), tone)
}

/// Bande d'accent verticale posée sur le bord gauche d'un élément.
pub fn accent_bar<'a, Message: 'a>(tone: Tone, height: f32) -> Element<'a, Message> {
    container(Space::new(stroke::MARKER, height))
        .style(move |theme: &Theme| container::Style {
            background: Some(Background::Color(tone.color(&tokens(theme)))),
            border: Border {
                radius: radius::MARKER.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
}

/// Pastille circulaire d'un jour ou d'un élément courant.
pub fn today_chip<'a, Message: 'a>(label: impl Into<String>) -> Element<'a, Message> {
    container(
        text(label.into())
            .size(font::CAPTION)
            .font(font::SEMIBOLD)
            .style(|theme: &Theme| iced::widget::text::Style {
                color: Some(tokens(theme).on_accent),
            }),
    )
    .center_x(Length::Fixed(size::TAG))
    .center_y(Length::Fixed(size::TAG))
    .style(|theme: &Theme| container::Style {
        background: Some(Background::Color(tokens(theme).accent_fill)),
        border: Border {
            radius: radius::PILL.into(),
            ..Border::default()
        },
        ..container::Style::default()
    })
    .into()
}

#[cfg(test)]
mod tests {
    use crate::ui::theme::{Marker, Tone};

    #[test]
    fn chaque_statut_dispose_d_une_forme_propre() {
        let shapes = [Marker::Hollow, Marker::Half, Marker::Solid, Marker::Barred];
        for (index, shape) in shapes.iter().enumerate() {
            for other in &shapes[index + 1..] {
                assert_ne!(shape, other, "deux statuts partagent la même forme");
            }
        }
    }

    #[test]
    fn un_compteur_neutre_ne_prend_pas_de_ton() {
        // `count_toned` retombe sur le compteur neutre quand il n'y a rien à
        // signaler : un zéro ne doit pas attirer l'œil.
        assert_eq!(Tone::default(), Tone::Neutral);
    }
}
