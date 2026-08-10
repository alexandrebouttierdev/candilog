//! Fond ambiant « glass » : halos radiaux sur le fond d'app, dessinés une
//! seule fois en arrière-plan (approximation native du `glass-ambient` CSS).
//!
//! Le canvas Iced 0.13 ne connaît que le dégradé linéaire (`gradient::Linear`) :
//! chaque halo radial est donc approximé par des cercles concentriques
//! d'opacité croissante, posés du plus grand (bord, discret) vers le plus petit
//! (centre, appuyé).

use crate::ui::theme::tokens::{tokens, Tokens};
use iced::widget::canvas::{self, Canvas, Fill, Frame, Path};
use iced::{Color, Element, Size, Theme};

/// Position et rayon d'un halo, en fractions de la fenêtre.
pub struct Halo {
    /// Centre relatif (0..1, 0..1).
    center: (f32, f32),
    /// Rayon relatif à la plus grande dimension.
    radius: f32,
    /// Couleur du dégradé radial.
    color: Color,
}

/// Cercles concentriques de chaque halo : plus il y en a, plus la chute
/// d'opacité est lisse (dessin unique, mis en cache par le `Canvas`).
const HALO_RINGS: usize = 24;

/// Halos du thème donné (deux : indigo haut-droite, bleu bas-gauche).
pub fn halos(palette: &Tokens) -> Vec<Halo> {
    if palette.is_dark {
        vec![
            Halo {
                center: (0.88, -0.14),
                radius: 0.62,
                color: Color {
                    a: 0.42,
                    ..Color {
                        r: 0.42,
                        g: 0.35,
                        b: 0.92,
                        a: 1.0,
                    }
                },
            },
            Halo {
                center: (-0.10, 1.12),
                radius: 0.55,
                color: Color {
                    a: 0.30,
                    ..Color {
                        r: 0.25,
                        g: 0.60,
                        b: 0.90,
                        a: 1.0,
                    }
                },
            },
        ]
    } else {
        vec![
            Halo {
                center: (0.88, -0.12),
                radius: 0.62,
                color: Color {
                    a: 0.30,
                    ..Color {
                        r: 0.55,
                        g: 0.49,
                        b: 0.95,
                        a: 1.0,
                    }
                },
            },
            Halo {
                center: (-0.08, 1.12),
                radius: 0.55,
                color: Color {
                    a: 0.26,
                    ..Color {
                        r: 0.42,
                        g: 0.72,
                        b: 0.95,
                        a: 1.0,
                    }
                },
            },
        ]
    }
}

/// Stops de dégradé par halo, exposés pour les tests.
pub fn halo_stops(palette: &Tokens) -> Vec<(f32, Color)> {
    halos(palette)
        .into_iter()
        .map(|halo| (0.0, halo.color))
        .collect()
}

/// Arrière-plan ambiant : fond d'app + halos radiaux.
pub struct Ambient;

impl<Message> canvas::Program<Message> for Ambient {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<iced::Renderer>> {
        let palette = tokens(theme);
        let mut frame = Frame::new(renderer, bounds.size());
        frame.fill_rectangle(
            iced::Point::ORIGIN,
            bounds.size(),
            Fill::from(palette.canvas),
        );
        let long_edge = bounds.size().width.max(bounds.size().height);
        for halo in halos(&palette) {
            let center = iced::Point::new(
                bounds.size().width * halo.center.0,
                bounds.size().height * halo.center.1,
            );
            let radius = long_edge * halo.radius;
            for ring in 0..HALO_RINGS {
                let ring_radius = radius * (HALO_RINGS - ring) as f32 / HALO_RINGS as f32;
                let ring_alpha = halo.color.a * (ring + 1) as f32 / HALO_RINGS as f32;
                frame.fill(
                    &Path::circle(center, ring_radius),
                    Fill::from(Color {
                        a: ring_alpha,
                        ..halo.color
                    }),
                );
            }
        }
        vec![frame.into_geometry()]
    }
}

/// Widget du fond ambiant, à poser sous le contenu dans un `stack`.
pub fn ambient<'a, Message: 'a>() -> Element<'a, Message> {
    Canvas::new(Ambient)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}

/// Taille de référence utilisée par le rendu.
pub fn reference_size(size: Size) -> Size {
    Size::new(size.width.max(1.0), size.height.max(1.0))
}

#[cfg(test)]
mod tests {
    use super::halo_stops;
    use crate::ui::theme::tokens::{DAY, NIGHT};

    #[test]
    fn les_halos_sont_deux_par_theme() {
        assert_eq!(halo_stops(&DAY).len(), 2);
        assert_eq!(halo_stops(&NIGHT).len(), 2);
    }

    #[test]
    fn le_fond_sombre_reste_plus_fonce_que_le_clair() {
        let nuit = halo_stops(&NIGHT);
        let jour = halo_stops(&DAY);
        assert!(nuit[0].1.a > 0.3, "halo sombre pas assez présent");
        assert!(
            jour[0].1.a < nuit[0].1.a,
            "halo clair plus appuyé que le sombre"
        );
    }
}
