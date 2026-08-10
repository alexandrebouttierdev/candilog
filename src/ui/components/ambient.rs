//! Fond ambiant « glass » : halos radiaux sur le fond d'app, dessinés une
//! seule fois en arrière-plan (approximation native du `glass-ambient` CSS).
//!
//! Le canvas Iced 0.13 ne connaît que le dégradé linéaire (`gradient::Linear`) :
//! chaque halo radial est donc approximé par des cercles concentriques posés du
//! plus grand (bord, discret) vers le plus petit (centre, appuyé). Les alphas
//! des anneaux sont calculés par `ring_alphas` pour que l'opacité visible
//! retombe linéairement du centre (`a`) au bord (`a/n`), sans cumul excessif
//! dû au mélange source-over : chaque anneau apporte exactement `a/n`.

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

/// Alphas incrémentaux des anneaux d'un halo pour une chute d'opacité
/// linéaire exacte : l'anneau k (indice `k` du vecteur, rayon `R·(n−k)/n`,
/// dessiné en premier, du plus grand rayon vers le plus petit) reçoit
/// `alpha_k = (a/n)/(1 − a·k/n)`.
///
/// La transparence cumulée après les anneaux `0..=k` est
/// `∏ (1 − alpha_j) = (1 − a·(k+1)/n)/(1 − 0) = 1 − a·(k+1)/n` (télescopage),
/// soit une opacité visible `a·(k+1)/n` sur l'anneau k : le centre atteint
/// exactement `a`, le bord `a/n ≈ 0`, chaque anneau ajoute `a/n` d'opacité.
/// Les alphas croissent du bord vers le centre (0.0175 → 0.0293 pour
/// a = 0.42), tous dans (0, 1).
fn ring_alphas(target_alpha: f32, rings: usize) -> Vec<f32> {
    let step = target_alpha / rings as f32;
    (0..rings)
        .map(|k| step / (1.0 - target_alpha * k as f32 / rings as f32))
        .collect()
}

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
            let alphas = ring_alphas(halo.color.a, HALO_RINGS);
            for (ring, alpha) in alphas.iter().enumerate() {
                let ring_radius = radius * (HALO_RINGS - ring) as f32 / HALO_RINGS as f32;
                frame.fill(
                    &Path::circle(center, ring_radius),
                    Fill::from(Color {
                        a: *alpha,
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
    use super::{halo_stops, ring_alphas};
    use crate::ui::theme::tokens::{DAY, NIGHT};

    #[test]
    fn les_alphas_des_anneaux_atteignent_la_cible_lineaire() {
        let alphas = ring_alphas(0.42, 24);
        assert_eq!(alphas.len(), 24);
        // transparence cumulée au centre = 1 − 0.42
        let centre: f32 = alphas.iter().map(|a| 1.0 - a).product();
        assert!((centre - 0.58).abs() < 1e-3);
        // tous dans (0,1)
        assert!(alphas.iter().all(|a| *a > 0.0 && *a < 1.0));
        // monotonie du premier au dernier
        assert!(alphas.windows(2).all(|w| w[0] < w[1]));
    }

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
