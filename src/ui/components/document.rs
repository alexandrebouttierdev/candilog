//! Aperçu de document posé dans un plan de travail.
//!
//! La page garde les proportions A4 et son propre encrage : elle ne suit pas
//! le thème de l'application, exactement comme une feuille posée sur un bureau.

use super::surface;
use super::typo;
use crate::ui::theme::metrics::{radius, space, stroke, A4_RATIO};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use iced::widget::{column, container, horizontal_rule, row, rule, text, Space};
use iced::{Alignment, Background, Border, Element, Length, Shadow, Theme, Vector};

/// Largeur de référence d'une page affichée à 100 %.
pub const BASE_WIDTH: f32 = 500.0;

/// Facteurs de zoom proposés par le plan de travail.
pub const ZOOM_LEVELS: [f32; 4] = [0.8, 1.0, 1.2, 1.4];

/// Largeur de page par défaut.
pub const DEFAULT_WIDTH: f32 = BASE_WIDTH;

/// Largeurs de page correspondant aux facteurs de zoom.
#[must_use]
pub fn zoom_widths() -> [f32; 4] {
    ZOOM_LEVELS.map(|level| BASE_WIDTH * level)
}

/// Pourcentage affiché pour une largeur de page.
#[must_use]
pub fn zoom_percent(width: f32) -> u32 {
    (width / BASE_WIDTH * 100.0).round() as u32
}

/// Hauteur d'une page A4 pour une largeur donnée.
#[must_use]
pub fn page_height(width: f32) -> f32 {
    width * A4_RATIO
}

/// Plan de travail sombre dans lequel la page est centrée et défile.
pub fn workspace<'a, Message: 'a>(page: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(
        surface::scroll(
            container(page.into())
                .center_x(Length::Fill)
                .padding(space::MAX),
        )
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|theme: &Theme| {
        let palette = tokens(theme);
        container::Style {
            background: Some(Background::Color(if palette.is_dark {
                palette.chrome
            } else {
                palette.hover
            })),
            text_color: Some(palette.text),
            ..container::Style::default()
        }
    })
    .into()
}

/// Page A4 : surface papier, ombre unique, largeur pilotée par le zoom.
pub fn page<'a, Message: 'a>(
    width: f32,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(content.into())
        .width(width)
        .height(page_height(width))
        .padding([width * 0.09, width * 0.085])
        .style(|theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(palette.paper)),
                text_color: Some(palette.paper_ink),
                border: Border {
                    color: palette.paper_rule,
                    width: stroke::HAIRLINE,
                    radius: radius::DOCUMENT.into(),
                },
                shadow: Shadow {
                    color: palette.shadow,
                    offset: Vector::new(0.0, 8.0),
                    blur_radius: 22.0,
                },
            }
        })
        .into()
}

/// Page A4 sans padding imposé, destinée aux templates qui portent leurs propres marges.
///
/// Le CV possède un header et un corps aux espacements distincts dans le HTML de référence ;
/// un padding uniforme autour du widget rendait toute correspondance pixel perfect impossible.
pub fn page_unpadded<'a, Message: 'a>(
    width: f32,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(content.into())
        .width(width)
        .height(page_height(width))
        .style(styles::document_paper)
        .into()
}

/// Titre de la page du document (nom du candidat, objet de la lettre).
pub fn heading<'a, Message: 'a>(value: impl Into<String>) -> Element<'a, Message> {
    text(value.into())
        .size(20.0)
        .font(font::SEMIBOLD)
        .style(|theme: &Theme| iced::widget::text::Style {
            color: Some(tokens(theme).paper_ink),
        })
        .into()
}

/// Sous-titre du document.
pub fn subheading<'a, Message: 'a>(value: impl Into<String>) -> Element<'a, Message> {
    text(value.into())
        .size(12.0)
        .font(font::MEDIUM)
        .style(|theme: &Theme| iced::widget::text::Style {
            color: Some(tokens(theme).paper_ink_muted),
        })
        .into()
}

/// Titre de rubrique à l'intérieur du document.
pub fn rubric<'a, Message: 'a>(value: impl Into<String>) -> Element<'a, Message> {
    column![
        text(value.into())
            .size(10.5)
            .font(font::SEMIBOLD)
            .style(|theme: &Theme| iced::widget::text::Style {
                color: Some(tokens(theme).paper_ink),
            }),
        horizontal_rule(1).style(|theme: &Theme| rule::Style {
            color: tokens(theme).paper_rule,
            width: 1,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
        }),
    ]
    .spacing(3)
    .into()
}

/// Corps de texte du document.
pub fn body<'a, Message: 'a>(value: impl Into<String>) -> Element<'a, Message> {
    text(value.into())
        .size(9.5)
        .style(|theme: &Theme| iced::widget::text::Style {
            color: Some(tokens(theme).paper_ink),
        })
        .into()
}

/// Texte secondaire du document.
pub fn body_muted<'a, Message: 'a>(value: impl Into<String>) -> Element<'a, Message> {
    text(value.into())
        .size(9.0)
        .style(|theme: &Theme| iced::widget::text::Style {
            color: Some(tokens(theme).paper_ink_muted),
        })
        .into()
}

/// Ligne d'expérience : intitulé à gauche, période à droite.
pub fn entry<'a, Message: 'a>(
    title: impl Into<String>,
    period: impl Into<String>,
    description: impl Into<String>,
) -> Element<'a, Message> {
    column![
        row![
            text(title.into())
                .size(10.0)
                .font(font::MEDIUM)
                .style(|theme: &Theme| iced::widget::text::Style {
                    color: Some(tokens(theme).paper_ink),
                }),
            Space::with_width(Length::Fill),
            body_muted(period.into()),
        ]
        .align_y(Alignment::Center),
        body_muted(description.into()),
    ]
    .spacing(2)
    .into()
}

/// Bandeau de contrôle du plan de travail : zoom et actions du document.
pub fn workbench_bar<'a, Message: 'a>(
    label: &'a str,
    controls: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        row![
            typo::label(label),
            Space::with_width(Length::Fill),
            controls.into(),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .height(30)
    .padding([0.0, space::LG])
    .width(Length::Fill)
    .align_y(Alignment::Center)
    .into()
}

#[cfg(test)]
mod tests {
    use super::{page_height, zoom_percent, zoom_widths, DEFAULT_WIDTH};
    use crate::ui::theme::metrics::A4_RATIO;

    #[test]
    fn la_page_respecte_les_proportions_a4() {
        for width in zoom_widths() {
            let ratio = page_height(width) / width;
            assert!((ratio - A4_RATIO).abs() < 0.001);
        }
    }

    #[test]
    fn les_paliers_de_zoom_sont_croissants() {
        assert!(zoom_widths().windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn la_largeur_par_defaut_appartient_aux_paliers() {
        assert!(zoom_widths().contains(&DEFAULT_WIDTH));
    }

    #[test]
    fn les_pourcentages_de_zoom_sont_lisibles() {
        let percents: Vec<u32> = zoom_widths().into_iter().map(zoom_percent).collect();
        assert_eq!(percents, vec![80, 100, 120, 140]);
    }

    #[test]
    fn la_page_la_plus_large_reste_affichable() {
        let widths = zoom_widths();
        let widest = widths[widths.len() - 1];
        assert!(widest <= 700.0, "page trop large pour un volet de travail");
    }
}
