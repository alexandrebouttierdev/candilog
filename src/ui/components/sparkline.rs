//! Mini-graphique en barres (activité 7 jours du tableau de bord).

use crate::ui::theme::tokens::tokens;
use iced::border;
use iced::widget::canvas::{self, Canvas, Fill, Frame, Path};
use iced::{Element, Length, Point, Rectangle, Renderer, Size, Theme};

/// Hauteurs relatives (%) des barres, au plancher de 4 % quand le max est nul.
#[must_use]
pub fn bar_heights(counts: [usize; 7]) -> [f32; 7] {
    let max = counts.iter().copied().max().unwrap_or(0);
    if max == 0 {
        return [4.0; 7];
    }
    let mut heights = [0.0; 7];
    for (index, count) in counts.iter().enumerate() {
        heights[index] = (*count as f32 / max as f32 * 100.0).max(4.0);
    }
    heights
}

struct Bars {
    heights: [f32; 7],
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
        let gap = bounds.width * 0.08;
        let bar_width = (bounds.width - gap * 6.0) / 7.0;
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

/// Mini-graphique de 7 barres avec valeurs et jours (rendus par l'appelant).
pub fn bar_chart<'a, Message: 'a>(heights: [f32; 7]) -> Element<'a, Message> {
    Canvas::new(Bars { heights })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
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
}
