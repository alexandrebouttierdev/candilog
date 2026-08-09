//! Barre latérale de navigation.
//!
//! Compacte (208 px), groupée, sans emoji. L'élément actif se signale par une
//! barre d'accent, un fond de sélection discret et un libellé en texte
//! principal : jamais par un gros bouton bordé.

use super::badge;
use super::icon::{self, Ink};
use super::typo;
use crate::navigation::{Route, Section};
use crate::ui::theme::metrics::{size, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use iced::widget::{button, column, container, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Charge à traiter affichée en regard d'un écran.
#[derive(Debug, Clone, Copy, Default)]
pub struct Load {
    /// Candidatures en attente d'action.
    pub candidatures: usize,
    /// Événements à venir dans l'agenda.
    pub agenda: usize,
    /// Relances arrivées à échéance.
    pub relances: usize,
}

impl Load {
    /// Compteur affiché pour un écran donné, s'il en mérite un.
    #[must_use]
    pub const fn badge_for(self, route: Route) -> Option<usize> {
        match route {
            Route::Candidatures if self.candidatures > 0 => Some(self.candidatures),
            Route::Calendrier if self.agenda > 0 => Some(self.agenda),
            _ => None,
        }
    }
}

/// Construit la navigation principale et le pied de barre latérale.
pub fn sidebar<'a, Message: Clone + 'a>(
    active: Route,
    load: Load,
    on_navigate: impl Fn(Route) -> Message + Copy + 'a,
    on_toggle_theme: Message,
    is_dark: bool,
) -> Element<'a, Message> {
    let brand = row![
        icon::brand(22.0),
        column![
            typo::item_strong("Candilog"),
            typo::caption(format!("v{}", env!("CARGO_PKG_VERSION"))),
        ]
        .spacing(0),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center);

    let mut nav = column![].spacing(1).padding([space::SM, space::MD]);
    for section in Section::ALL {
        if let Some(label) = section.label() {
            nav = nav.push(
                container(typo::caption(label))
                    .height(24)
                    .padding([0.0, space::SM])
                    .align_y(Alignment::Center),
            );
        }
        for route in Route::of_section(section) {
            nav = nav.push(entry(route, route == active, load, on_navigate));
        }
    }

    let footer = container(
        row![
            typo::caption(if is_dark { "Nuit" } else { "Jour" }),
            Space::with_width(Length::Fill),
            super::button::icon_action(
                if is_dark {
                    icon::Icon::Sun
                } else {
                    icon::Icon::Moon
                },
                "Changer d'apparence",
                on_toggle_theme,
            ),
        ]
        .align_y(Alignment::Center),
    )
    .padding([0.0, space::MD]);

    container(
        column![
            container(brand)
                .height(size::TOOLBAR)
                .padding([0.0, space::XL]),
            super::surface::divider(),
            super::surface::scroll(nav).height(Length::Fill),
            super::surface::divider(),
            container(footer).height(34).align_y(Alignment::Center),
        ]
        .height(Length::Fill),
    )
    .width(size::SIDEBAR)
    .height(Length::Fill)
    .style(|theme: &Theme| {
        let palette = tokens(theme);
        container::Style {
            background: Some(Background::Color(palette.chrome)),
            text_color: Some(palette.text),
            border: Border {
                color: palette.border,
                width: stroke::HAIRLINE,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        }
    })
    .into()
}

fn entry<'a, Message: Clone + 'a>(
    route: Route,
    active: bool,
    load: Load,
    on_navigate: impl Fn(Route) -> Message,
) -> Element<'a, Message> {
    let trailing: Element<'a, Message> = match load.badge_for(route) {
        Some(count) => badge::count(count),
        None => Space::with_width(0).into(),
    };
    let content = row![
        marker(active),
        icon::icon(
            route.icon(),
            icon::MD,
            if active { Ink::Accent } else { Ink::Muted },
        ),
        typo::body(route.label()),
        Space::with_width(Length::Fill),
        trailing,
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center);

    button(content)
        .width(Length::Fill)
        .height(size::NAV_ROW)
        .padding([0.0, space::MD])
        .style(styles::nav_item(active))
        .on_press(on_navigate(route))
        .into()
}

fn marker<'a, Message: 'a>(active: bool) -> Element<'a, Message> {
    container(Space::new(stroke::MARKER, 14.0))
        .style(move |theme: &Theme| container::Style {
            background: active.then(|| Background::Color(tokens(theme).accent)),
            border: Border {
                radius: 1.0.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::Load;
    use crate::navigation::Route;

    #[test]
    fn un_compteur_nul_n_est_pas_affiche() {
        let load = Load::default();
        for route in Route::ALL {
            assert_eq!(load.badge_for(route), None);
        }
    }

    #[test]
    fn seuls_les_ecrans_porteurs_de_charge_affichent_un_compteur() {
        let load = Load {
            candidatures: 4,
            agenda: 2,
            relances: 3,
        };
        assert_eq!(load.badge_for(Route::Candidatures), Some(4));
        assert_eq!(load.badge_for(Route::Calendrier), Some(2));
        assert_eq!(load.badge_for(Route::Statistiques), None);
        assert_eq!(load.badge_for(Route::Parametres), None);
    }
}
