//! Pagination compacte : boutons Précédent/Suivant et fenêtre « x–y sur n ».

use super::button as controls;
use super::icon::Icon;
use super::typo;
use crate::ui::theme::metrics::space;
use iced::widget::{row, Space};
use iced::{Alignment, Element, Length};

/// Fenêtre affichée « x–y » pour la page courante (1-based), bornée au total.
#[must_use]
pub fn window(page: u64, page_size: u64, total: u64) -> (u64, u64) {
    if total == 0 {
        return (0, 0);
    }
    let total_pages = total.div_ceil(page_size);
    let page = page.clamp(1, total_pages);
    let first = (page - 1) * page_size + 1;
    let last = (page * page_size).min(total);
    (first, last)
}

/// Contrôles Précédent/Suivant + compteur « x–y sur n ».
pub fn pagination<'a, Message: Clone + 'a>(
    _page: u64,
    _total_pages: u64,
    on_prev: Message,
    on_next: Message,
    first: u64,
    last: u64,
    total: u64,
) -> Element<'a, Message> {
    row![
        controls::ghost("Précédent", Some(Icon::ArrowLeft))
            .on_press(on_prev)
            .width(Length::Shrink),
        Space::with_width(Length::Fill),
        typo::caption(format!("{first}–{last} sur {total}")),
        controls::ghost("Suivant", Some(Icon::ArrowRight))
            .on_press(on_next)
            .width(Length::Shrink),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center)
    .padding([0.0, space::LG])
    .into()
}

#[cfg(test)]
mod tests {
    use super::window;

    #[test]
    fn la_fenetre_est_bornee_par_le_total() {
        assert_eq!(window(1, 20, 45), (1, 20));
        assert_eq!(window(3, 20, 45), (41, 45));
        assert_eq!(window(1, 20, 0), (0, 0));
    }

    #[test]
    fn la_page_est_clampee_aux_bornes() {
        assert_eq!(window(0, 20, 45), (1, 20));
        assert_eq!(window(9, 20, 45), (41, 45));
    }
}
