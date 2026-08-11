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
    let page_size = if page_size == 0 { total } else { page_size };
    let total_pages = total.div_ceil(page_size);
    let page = page.clamp(1, total_pages);
    let first = (page - 1) * page_size + 1;
    let last = (page * page_size).min(total);
    (first, last)
}

/// Contrôles Précédent/Suivant + rang de page et compteur « x–y sur n ».
///
/// `page` et `total_pages` étaient reçus mais **jamais utilisés** : les deux boutons portaient
/// toujours un `on_press`, donc restaient visuellement actifs aux deux extrémités, et le rang
/// de la page n'apparaissait nulle part. Un `on_press` retiré rend le bouton inerte *et*
/// grisé — c'est le mécanisme d'état désactivé d'Iced.
pub fn pagination<'a, Message: Clone + 'a>(
    page: u64,
    total_pages: u64,
    on_prev: Message,
    on_next: Message,
    first: u64,
    last: u64,
    total: u64,
) -> Element<'a, Message> {
    let mut precedent = controls::ghost("Précédent", Some(Icon::ArrowLeft)).width(Length::Shrink);
    if page > 1 {
        precedent = precedent.on_press(on_prev);
    }
    let mut suivant = controls::ghost("Suivant", Some(Icon::ArrowRight)).width(Length::Shrink);
    if page < total_pages {
        suivant = suivant.on_press(on_next);
    }
    let rang = if total_pages <= 1 {
        format!("{first}–{last} sur {total}")
    } else {
        format!("Page {page} / {total_pages} — {first}–{last} sur {total}")
    };
    row![
        precedent,
        Space::with_width(Length::Fill),
        typo::caption(rang),
        suivant,
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center)
    .padding([0.0, space::LG])
    .into()
}

/// Nombre de pages nécessaires pour `total` éléments de `page_size`, au minimum 1.
///
/// Sert à borner les transitions d'état : la couche d'affichage clampait déjà, mais le
/// compteur, lui, croissait sans limite — dix clics sur « Suivant » au bout d'un historique de
/// trois pages portaient `ats_page` à 13, et il fallait ensuite dix clics sur « Précédent »
/// pour que la pagination réagisse de nouveau.
#[must_use]
pub const fn total_pages(total: u64, page_size: u64) -> u64 {
    if page_size == 0 || total == 0 {
        return 1;
    }
    total.div_ceil(page_size)
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

    #[test]
    fn une_taille_de_page_nulle_ne_panique_pas() {
        assert_eq!(window(1, 0, 45), (1, 45));
    }
}
