//! Barre supérieure : onglets contextuels, recherche globale et état du runtime.
//!
//! La barre suit la maquette « refonte-design » : les onglets de l'espace actif à
//! gauche, la recherche globale au centre-droit et l'état du fournisseur IA à
//! l'extrême droite.

use crate::app::{App, Message};
use crate::ui::components::field;
use crate::ui::components::sidebar::workspace_tab_controls;
use crate::ui::theme::metrics::{size, space};
use crate::ui::theme::tokens::tokens;
use iced::widget::{container, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Barre supérieure : onglets de l'espace actif, recherche globale et runtime.
pub fn titlebar<'a>(app: &'a App, runtime: Element<'a, Message>) -> Element<'a, Message> {
    let tabs = workspace_tab_controls(app.route, Message::Navigate);
    let search = field::search_resettable(
        "Rechercher…",
        &app.search,
        Message::SearchChanged,
        Message::ResetSearch,
        Length::Fixed(240.0),
        Some(crate::app::SEARCH_FIELD_ID),
    );
    container(
        row![
            tabs,
            Space::with_width(Length::Fill),
            search,
            Space::with_width(space::MD),
            runtime,
        ]
        .align_y(Alignment::Center)
        .padding([0.0, space::LG]),
    )
    .width(Length::Fill)
    .height(size::TOPBAR)
    .align_y(Alignment::Center)
    .style(|theme: &Theme| {
        let palette = tokens(theme);
        container::Style {
            background: Some(Background::Color(palette.panel)),
            border: Border {
                color: palette.border,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        }
    })
    .into()
}

#[cfg(test)]
mod tests {
    use crate::ui::theme::metrics::size;

    #[test]
    fn la_barre_superieure_fait_46_pixels() {
        assert_eq!(size::TOPBAR, 46.0);
    }
}
