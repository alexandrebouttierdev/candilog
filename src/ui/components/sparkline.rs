//! Mini-graphique en barres (activité du tableau de bord et des statistiques).

use crate::ui::theme::tokens::tokens;
use iced::border;
use iced::widget::canvas::{self, Canvas, Fill, Frame, Path};
use iced::{Element, Length, Point, Rectangle, Renderer, Size, Theme};

/// Hauteurs relatives (%) des barres, au plancher de 4 % quand le max est nul.
#[must_use]
pub fn bar_heights<const N: usize>(counts: [usize; N]) -> [f32; N] {
    let max = counts.iter().copied().max().unwrap_or(0);
    if max == 0 {
        return [4.0; N];
    }
    let mut heights = [0.0; N];
    for (index, count) in counts.iter().enumerate() {
        heights[index] = (*count as f32 / max as f32 * 100.0).max(4.0);
    }
    heights
}

struct Bars {
    heights: Vec<f32>,
}

impl<Message> canvas::Program<Message> for Bars {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let palette = tokens(theme);
        let mut frame = Frame::new(renderer, bounds.size());
        let count = self.heights.len();
        if count == 0 {
            return vec![frame.into_geometry()];
        }
        let gap = bounds.width * 0.08;
        let bar_width = ((bounds.width - gap * (count as f32 - 1.0)) / count as f32).max(1.0);
        let radius = 4.0_f32.min(bar_width / 2.0);
        for (index, height) in self.heights.iter().enumerate() {
            let h = bounds.height * height / 100.0;
            let x = index as f32 * (bar_width + gap);
            let y = bounds.height - h;
            // Barre arrondie en haut (équivalent `rounded-t-md`).
            let path = Path::new(|builder| {
                builder.rounded_rectangle(
                    Point::new(x, y),
                    Size::new(bar_width, h),
                    border::top(radius),
                );
            });
            let mut color = palette.accent;
            color.a = 0.80;
            frame.fill(&path, Fill::from(color));
        }
        vec![frame.into_geometry()]
    }
}

/// Mini-graphique d'un nombre quelconque de barres (valeurs rendues par l'appelant).
pub fn bar_chart_n<'a, Message: 'a>(heights: &[f32]) -> Element<'a, Message> {
    Canvas::new(Bars {
        heights: heights.to_vec(),
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Mini-graphique de 7 barres, cas du tableau de bord.
pub fn bar_chart<'a, Message: 'a>(heights: [f32; 7]) -> Element<'a, Message> {
    bar_chart_n(&heights)
}

#[cfg(test)]
mod tests {
    use super::bar_heights;

    #[test]
    fn les_hauteurs_sont_proportionnelles_au_max() {
        let heights = bar_heights([0, 2, 4, 2, 0, 1, 3]);
        assert_eq!(heights[2], 100.0);
        assert_eq!(heights[1], 50.0);
        assert_eq!(heights[0], 4.0, "plancher de visibilité");
    }

    #[test]
    fn un_max_nul_donne_des_barres_au_plancher() {
        let heights = bar_heights([0, 0, 0, 0, 0, 0, 0]);
        assert!(heights.iter().all(|h| *h == 4.0));
    }

    #[test]
    fn les_hauteurs_se_calculent_aussi_pour_huit_barres() {
        let heights = bar_heights([0, 1, 2, 3, 0, 5, 0, 1]);
        assert_eq!(heights.len(), 8);
        assert_eq!(heights[5], 100.0);
        assert_eq!(heights[0], 4.0, "plancher de visibilité");
    }
}
