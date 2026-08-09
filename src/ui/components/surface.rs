//! Surfaces et séparateurs : la structure sans les cartes.
//!
//! Un panneau ne contient jamais un autre panneau. Pour hiérarchiser à
//! l'intérieur d'un panneau, on utilise un creux (`sunken`) ou un filet
//! (`divider`).

use super::typo;
use crate::ui::theme::metrics::{radius, size, space};
use crate::ui::theme::styles;
use iced::widget::{column, container, horizontal_rule, row, scrollable, vertical_rule, Container};
use iced::{Alignment, Element, Length};

/// Panneau de données : surface, filet, rayon.
pub fn panel<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    container(content).padding(space::XL).style(styles::panel)
}

/// Panneau sans padding, quand il héberge une liste bord à bord.
pub fn panel_bare<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Container<'a, Message> {
    container(content).style(styles::panel)
}

/// Zone de premier niveau accolée aux bords de la fenêtre, sans rayon.
pub fn region<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    container(content).style(styles::panel_flat)
}

/// Creux intégré à un panneau : encadré secondaire, résumé, bloc de réglage.
pub fn sunken<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    container(content).padding(space::LG).style(styles::sunken)
}

/// Surface surélevée : menu, feuille, panneau flottant.
pub fn raised<'a, Message: 'a>(content: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    container(content).padding(space::XL).style(styles::raised)
}

/// Filet horizontal intérieur à un panneau.
pub fn divider<'a, Message: 'a>() -> Element<'a, Message> {
    horizontal_rule(1).style(styles::divider).into()
}

/// Filet horizontal appuyé, sous un en-tête.
pub fn divider_strong<'a, Message: 'a>() -> Element<'a, Message> {
    horizontal_rule(1).style(styles::divider_strong).into()
}

/// Filet vertical séparant deux volets.
pub fn split_rule<'a, Message: 'a>() -> Element<'a, Message> {
    vertical_rule(1).style(styles::divider).into()
}

/// En-tête d'une section interne : titre, métadonnée, actions.
pub fn section_header<'a, Message: 'a>(
    title: &'a str,
    trailing: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        row![
            typo::section(title),
            iced::widget::Space::with_width(Length::Fill),
            trailing.into(),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .height(size::SECTION_HEADER)
    .align_y(Alignment::Center)
    .into()
}

/// Section complète : en-tête, filet, contenu.
pub fn section<'a, Message: 'a>(
    title: &'a str,
    trailing: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        section_header(title, trailing),
        divider(),
        container(content).padding([space::LG, 0.0]),
    ]
    .into()
}

/// Zone défilante verticale au style Candilog.
pub fn scroll<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> scrollable::Scrollable<'a, Message> {
    scrollable(content)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new()
                .width(6)
                .scroller_width(5)
                .margin(1),
        ))
        .style(styles::scroller)
}

/// Zone défilante horizontale au style Candilog.
pub fn scroll_x<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> scrollable::Scrollable<'a, Message> {
    // `with_direction` valide l'axe demandé ; `scrollable(…).direction(…)`
    // validerait d'abord l'axe vertical par défaut et rejetterait un contenu
    // qui occupe volontairement toute la hauteur, comme un board Kanban.
    scrollable::Scrollable::with_direction(
        content,
        scrollable::Direction::Horizontal(
            scrollable::Scrollbar::new()
                .width(6)
                .scroller_width(5)
                .margin(1),
        ),
    )
    .style(styles::scroller)
}

/// Rayon appliqué aux surfaces flottantes, exposé pour les compositions ad hoc.
pub const FLOATING_RADIUS: f32 = radius::DIALOG;
