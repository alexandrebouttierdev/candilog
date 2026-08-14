//! Sélecteur de relation (entité liée) : recherche, sélecteur et pagination
//! compacte. Composant partagé par les formulaires métier, indépendant de
//! l'état global : la taille de page est reçue en paramètre.

use crate::ui::components::{field, pagination};
use crate::ui::theme::metrics::space;
use iced::widget::column;
use iced::Element;

/// Paramètres de navigation de la liste paginée d'une relation.
#[derive(Debug, Clone)]
pub struct RelationNavigation<Message> {
    pub page: u64,
    pub total_pages: u64,
    pub total: u64,
    pub previous: Message,
    pub next: Message,
}

/// Champ de relation : recherche, sélecteur et pagination compacte.
pub fn relation_page<'a, Message: Clone + 'a>(
    search: &'a str,
    on_search: impl Fn(String) -> Message + 'a,
    page_size: u64,
    navigation: RelationNavigation<Message>,
    select: Element<'a, Message>,
) -> Element<'a, Message> {
    let (first, last) = pagination::window(navigation.page, page_size, navigation.total);
    column![
        field::search("Rechercher…", search, on_search, iced::Length::Fill),
        select,
        pagination::pagination(
            navigation.page,
            navigation.total_pages.max(1),
            navigation.previous,
            navigation.next,
            first,
            last,
            navigation.total,
        ),
    ]
    .spacing(space::SM)
    .into()
}
