//! Anneau de proportion (donut) : piste, arc coloré, valeur centrée.

use super::typo;
use crate::ui::theme::metrics::stroke;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::canvas::path::Arc;
use iced::widget::canvas::{self, Canvas, Frame, LineCap, Path, Stroke};
use iced::{Element, Length, Point, Radians, Rectangle, Theme};

/// Borne le ratio entre 0 et 1.
#[must_use]
pub fn ratio_borne(ratio: f32) -> f32 {
    ratio.clamp(0.0, 1.0)
}

struct Donut {
    ratio: f32,
    tone: Tone,
}

impl<Message> canvas::Program<Message> for Donut {
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
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let radius = (bounds.width - 18.0) / 2.0;
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

        let ratio = ratio_borne(self.ratio);
        if ratio > 0.0 {
            let start = std::f32::consts::FRAC_PI_2;
            let end = start - 2.0 * std::f32::consts::PI * ratio;
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
                .with_color(self.tone.color(&palette)),
            );
        }
        vec![frame.into_geometry()]
    }
}

/// Donut de la taille donnée ; le texte central est rendu par l'appelant.
pub fn donut<'a, Message: 'a>(ratio: f32, size: f32, tone: Tone) -> Element<'a, Message> {
    Canvas::new(Donut { ratio, tone })
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .into()
}

/// Valeur centrale du donut en mono semibold.
pub fn center<'a>(text: impl Into<String>) -> iced::widget::Text<'a> {
    typo::text_mono(text.into(), 20.0, font::MONO_SEMIBOLD)
}

#[cfg(test)]
mod tests {
    use super::ratio_borne;

    #[test]
    fn le_ratio_est_borne_entre_zero_et_un() {
        assert_eq!(ratio_borne(-0.5), 0.0);
        assert_eq!(ratio_borne(1.5), 1.0);
        assert_eq!(ratio_borne(0.42), 0.42);
    }
}
