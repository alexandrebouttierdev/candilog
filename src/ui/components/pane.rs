//! Volet listant les écrans du groupe actif.
//!
//! Il se place entre le rail et le contenu. Un groupe qui n'a qu'un écran
//! n'ouvre pas de volet : le contenu récupère sa largeur.

use super::rail::Load;
use super::typo;
use crate::navigation::{Route, Section};
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::tokens::tokens;
use iced::widget::{button, column, container, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Vrai si le groupe compte assez d'écrans pour mériter un volet.
fn has_pane(section: Section) -> bool {
    Route::of_section(section).len() > 1
}

/// Construit le volet du groupe actif, ou rien si le groupe n'a qu'un écran.
pub fn pane<'a, Message: Clone + 'a>(
    section: Section,
    active: Route,
    load: Load,
    on_navigate: impl Fn(Route) -> Message + Copy + 'a,
) -> Option<Element<'a, Message>> {
    if !has_pane(section) {
        return None;
    }

    let mut list = column![].spacing(space::XXS);
    for route in Route::of_section(section) {
        list = list.push(entry(route, route == active, load, on_navigate));
    }

    let content = column![
        container(typo::label(section.short_label()))
            .height(size::PANE_ROW)
            .padding([0.0, space::MD])
            .align_y(Alignment::Center),
        list,
    ]
    .width(Length::Fill);

    Some(
        container(content)
            .width(Length::Fixed(size::PANE))
            .height(Length::Fill)
            .padding([space::MD, space::SM])
            .style(|theme: &Theme| {
                let palette = tokens(theme);
                container::Style {
                    background: Some(Background::Color(palette.canvas)),
                    text_color: Some(palette.text),
                    border: Border {
                        color: palette.border,
                        width: stroke::HAIRLINE,
                        radius: radius::NONE.into(),
                    },
                    ..container::Style::default()
                }
            })
            .into(),
    )
}

/// Une ligne du volet : marqueur, libellé, compteur.
fn entry<'a, Message: Clone + 'a>(
    route: Route,
    active: bool,
    load: Load,
    on_navigate: impl Fn(Route) -> Message,
) -> Element<'a, Message> {
    let trailing: Element<'a, Message> = match load.badge_for_route(route) {
        Some(count) => super::badge::count(count),
        None => Space::with_width(0).into(),
    };

    let content = row![
        marker(active),
        typo::body(route.label()),
        Space::with_width(Length::Fill),
        trailing,
    ]
    .spacing(space::SM)
    .align_y(Alignment::Center);

    button(content)
        .width(Length::Fill)
        .height(size::PANE_ROW)
        .padding([0.0, space::SM])
        .on_press(on_navigate(route))
        .style(move |theme: &Theme, status| {
            let palette = tokens(theme);
            let background = if active {
                Some(palette.selection)
            } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
                Some(palette.hover)
            } else {
                None
            };
            button::Style {
                background: background.map(Background::Color),
                text_color: if active {
                    palette.accent
                } else {
                    palette.text_secondary
                },
                border: Border {
                    color: iced::Color::TRANSPARENT,
                    width: 0.0,
                    radius: radius::CONTROL.into(),
                },
                shadow: iced::Shadow::default(),
            }
        })
        .into()
}

/// Marqueur de sélection, second signal indépendant de la couleur du texte.
fn marker<'a, Message: 'a>(active: bool) -> Element<'a, Message> {
    container(Space::new(stroke::MARKER, size::MARKER))
        .style(move |theme: &Theme| container::Style {
            background: active.then(|| Background::Color(tokens(theme).accent)),
            border: Border {
                radius: radius::MARKER.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use crate::navigation::{Route, Section};

    /// Un groupe à écran unique n'a rien à lister.
    #[test]
    fn un_groupe_a_ecran_unique_n_ouvre_pas_de_volet() {
        assert!(super::has_pane(Section::Recherche));
        assert!(!super::has_pane(Section::Pilotage));
    }

    /// La règle est dérivée du nombre d'écrans, jamais codée en dur sur un
    /// groupe : ajouter un écran à Pilotage doit suffire à lui donner un volet.
    #[test]
    fn la_regle_du_volet_suit_le_nombre_d_ecrans() {
        for section in Section::ALL {
            assert_eq!(
                super::has_pane(section),
                Route::of_section(section).len() > 1
            );
        }
    }
}
