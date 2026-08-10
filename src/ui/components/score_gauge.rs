//! Jauge annulaire du score ATS : piste, arc arrondi, valeur mono centrée.

use super::typo;
use crate::ui::theme::metrics::stroke;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::canvas::path::Arc;
use iced::widget::canvas::{self, Canvas, Frame, LineCap, Path, Stroke};
use iced::{Element, Length, Point, Radians, Rectangle, Theme};

/// Ton du score selon les seuils candilog-desktop.
#[must_use]
pub fn tone_pour_score(score: u8) -> Tone {
    if score >= 80 {
        Tone::Success
    } else if score >= 60 {
        Tone::Warning
    } else {
        Tone::Danger
    }
}

struct Gauge {
    score: u8,
    size: f32,
}

impl<Message> canvas::Program<Message> for Gauge {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<iced::Renderer>> {
        let palette = tokens(theme);
        let mut frame = Frame::new(renderer, bounds.size());
        let size = bounds.size();
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let radius = (self.size - 6.0) / 2.0;
        let ratio = (self.score as f32 / 100.0).clamp(0.0, 1.0);
        let start = std::f32::consts::FRAC_PI_2;
        let end = start - 2.0 * std::f32::consts::PI * ratio;
        let width = stroke::EMPHASIS * 2.0;

        let track = Path::circle(center, radius);
        frame.stroke(
            &track,
            Stroke {
                width,
                ..Stroke::default()
            }
            .with_color(palette.sunken),
        );

        if ratio > 0.0 {
            let arc = Path::new(|builder| {
                builder.arc(Arc {
                    center,
                    radius,
                    start_angle: Radians(start),
                    end_angle: Radians(end),
                });
            });
            frame.stroke(
                &arc,
                Stroke {
                    width,
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                }
                .with_color(tone_pour_score(self.score).color(&palette)),
            );
        }
        vec![frame.into_geometry()]
    }
}

/// Jauge de score ATS de la taille donnée (56 par défaut, 48 dans le générateur).
pub fn gauge<'a, Message: 'a>(score: u8, size: f32) -> Element<'a, Message> {
    Canvas::new(Gauge { score, size })
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .into()
}

/// Rendu d'un score ATS en chiffre mono.
pub fn score_label<'a>(score: u8) -> iced::widget::Text<'a> {
    typo::text_mono(score.to_string(), 15.0, font::MONO_SEMIBOLD)
}

#[cfg(test)]
mod tests {
    use super::tone_pour_score;
    use crate::ui::theme::Tone;

    #[test]
    fn un_score_elevé_est_un_succes() {
        assert_eq!(tone_pour_score(85), Tone::Success);
        assert_eq!(tone_pour_score(80), Tone::Success);
    }

    #[test]
    fn un_score_moyen_est_un_avertissement() {
        assert_eq!(tone_pour_score(70), Tone::Warning);
        assert_eq!(tone_pour_score(60), Tone::Warning);
    }

    #[test]
    fn un_score_faible_est_une_erreur() {
        assert_eq!(tone_pour_score(59), Tone::Danger);
        assert_eq!(tone_pour_score(0), Tone::Danger);
    }
}
