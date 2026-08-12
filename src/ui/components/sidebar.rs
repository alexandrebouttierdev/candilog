//! Rail de navigation desktop et onglets contextuels des espaces de travail.
//!
//! Le rail ne liste volontairement pas toutes les routes : il expose sept espaces
//! stables, à la manière d'un logiciel de création. Les routes d'un espace
//! restent toutes accessibles dans la bande d'onglets placée au-dessus de la vue.

use crate::navigation::{Route, Section};
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::typo;
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};

const BRAND_HEIGHT: f32 = 72.0;
const TILE_HEIGHT: f32 = 62.0;
const ICON_WELL: f32 = 31.0;
const CONTEXT_HEIGHT: f32 = 42.0;

/// Espaces du rail avec leur état actif, dans l'ordre visuel.
#[must_use]
pub fn rail_entries(active: Route) -> Vec<(Section, bool)> {
    Section::ALL
        .into_iter()
        .map(|section| (section, section == active.section()))
        .collect()
}

/// Rail vertical complet, marque et bascule de thème comprises.
pub fn sidebar<'a, Message: Clone + 'a>(
    active: Route,
    on_navigate: impl Fn(Route) -> Message + 'a,
    is_dark: bool,
    on_toggle_theme: Message,
    footer: Element<'a, Message>,
    on_about: Message,
) -> Element<'a, Message> {
    let mut navigation = column![]
        .spacing(space::XXS)
        .width(Length::Fill)
        .align_x(Alignment::Center);

    for (section, is_active) in rail_entries(active) {
        navigation = navigation.push(tile(
            section.icon(),
            section.short_label(),
            is_active,
            on_navigate(section.default_route()),
        ));
    }

    let brand = button(
        container(
            column![
                icon::brand(34.0),
                typo::body("Candilog").font(font::SEMIBOLD),
            ]
            .spacing(space::XS)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill),
    )
    .height(BRAND_HEIGHT)
    .width(Length::Fill)
    .padding(0)
    .style(styles::ghost)
    .on_press(on_about.clone());

    let footer = button(
        container(footer)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .height(20.0)
    .width(Length::Fill)
    .padding(0)
    .style(styles::ghost)
    .on_press(on_about);

    let theme = tile(
        if is_dark { Icon::Sun } else { Icon::Moon },
        if is_dark { "Clair" } else { "Sombre" },
        false,
        on_toggle_theme,
    );

    container(
        column![
            brand,
            container(navigation).padding([space::SM, space::XS]),
            Space::with_height(Length::Fill),
            container(theme).padding([space::XS, space::XS]),
            footer,
        ]
        .height(Length::Fill)
        .width(Length::Fill)
        .align_x(Alignment::Center),
    )
    .width(size::SIDEBAR)
    .height(Length::Fill)
    .style(rail_style)
    .into()
}

/// Bande d'onglets donnant accès aux routes de l'espace actif.
///
/// Les espaces à route unique n'ajoutent aucune hauteur à la coquille.
pub fn workspace_tabs<'a, Message: Clone + 'a>(
    active: Route,
    on_navigate: impl Fn(Route) -> Message + 'a,
) -> Element<'a, Message> {
    let routes = Route::of_section(active.section());
    if routes.len() <= 1 {
        return Space::with_height(0).into();
    }

    let tabs = workspace_tab_controls(active, on_navigate);

    container(tabs)
        .height(CONTEXT_HEIGHT)
        .width(Length::Fill)
        .padding([space::XS, space::LG])
        .align_y(Alignment::Center)
        .style(context_bar_style)
        .into()
}

/// Contrôles d'onglets sans bande englobante, pour une toolbar d'espace unifiée.
pub fn workspace_tab_controls<'a, Message: Clone + 'a>(
    active: Route,
    on_navigate: impl Fn(Route) -> Message + 'a,
) -> Element<'a, Message> {
    let mut tabs = row![].spacing(space::XS).align_y(Alignment::Center);
    let routes = Route::of_section(active.section());
    for route in routes {
        let selected = route == active;
        let control = button(crate::ui::components::button::vcenter(
            row![
                icon::icon(
                    route.icon(),
                    14.0,
                    if selected { Ink::Accent } else { Ink::Muted },
                ),
                text(route.label()).size(font::META).font(if selected {
                    font::MEDIUM
                } else {
                    font::REGULAR
                }),
            ]
            .spacing(space::SM)
            .align_y(Alignment::Center),
        ))
        .height(size::ACTION)
        .padding([0.0, space::MD])
        .style(move |theme: &Theme, status| context_tab_style(theme, status, selected))
        .on_press(on_navigate(route));
        tabs = tabs.push(control);
    }
    tabs.into()
}

fn tile<'a, Message: Clone + 'a>(
    glyph: Icon,
    label: &'static str,
    active: bool,
    on_press: Message,
) -> Element<'a, Message> {
    let icon_well = container(icon::icon(
        glyph,
        icon::MD,
        if active { Ink::OnAccent } else { Ink::Strong },
    ))
    .width(ICON_WELL)
    .height(ICON_WELL)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(move |theme: &Theme| icon_well_style(theme, active));

    let label = text(label)
        .size(font::MICRO)
        .font(if active { font::MEDIUM } else { font::REGULAR })
        .style(move |theme: &Theme| iced::widget::text::Style {
            color: Some(if active {
                tokens(theme).accent
            } else {
                tokens(theme).text_secondary
            }),
        });

    button(
        container(
            column![icon_well, label]
                .spacing(space::XS)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .height(TILE_HEIGHT)
    .padding(0)
    .style(move |theme: &Theme, status| tile_style(theme, status, active))
    .on_press(on_press)
    .into()
}

fn rail_style(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(palette.chrome)),
        text_color: Some(palette.text),
        border: Border {
            color: palette.border,
            width: stroke::HAIRLINE,
            radius: radius::NONE.into(),
        },
        ..container::Style::default()
    }
}

fn icon_well_style(theme: &Theme, active: bool) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(if active {
            palette.accent_fill
        } else {
            palette.panel
        })),
        border: Border {
            color: if active {
                palette.accent_fill
            } else {
                palette.border_strong
            },
            width: stroke::HAIRLINE,
            radius: radius::CONTROL.into(),
        },
        ..container::Style::default()
    }
}

fn tile_style(theme: &Theme, status: button::Status, active: bool) -> button::Style {
    let palette = tokens(theme);
    let background = if matches!(status, button::Status::Hovered | button::Status::Pressed) {
        Some(if active {
            palette.selection
        } else {
            alpha(palette.panel, 0.78)
        })
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
            radius: radius::CONTROL.into(),
            ..Border::default()
        },
        shadow: Shadow::default(),
    }
}

fn context_bar_style(theme: &Theme) -> container::Style {
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
}

fn context_tab_style(theme: &Theme, status: button::Status, active: bool) -> button::Style {
    let palette = tokens(theme);
    button::Style {
        background: if active {
            Some(Background::Color(palette.selection))
        } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
            Some(Background::Color(palette.hover))
        } else {
            None
        },
        text_color: if active {
            palette.accent
        } else {
            palette.text_secondary
        },
        border: Border {
            color: if active {
                alpha(palette.accent, 0.45)
            } else {
                Color::TRANSPARENT
            },
            width: stroke::HAIRLINE,
            radius: radius::CONTROL.into(),
        },
        shadow: Shadow::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::{rail_entries, sidebar, workspace_tab_controls, workspace_tabs};
    use crate::navigation::{Route, Section};

    #[test]
    fn le_rail_s_instancie() {
        let _: iced::Element<'_, ()> = sidebar(
            Route::Dashboard,
            |route| {
                let _ = route;
            },
            true,
            (),
            iced::widget::Space::with_width(0).into(),
            (),
        );
    }

    #[test]
    fn le_rail_affiche_tous_les_espaces_dont_un_actif() {
        let entries = rail_entries(Route::CvGenerator);
        assert_eq!(entries.len(), Section::ALL.len());
        assert_eq!(entries.iter().filter(|(_, active)| *active).count(), 1);
        assert!(entries.contains(&(Section::Documents, true)));
    }

    #[test]
    fn les_onglets_contextuels_s_instancient_pour_toutes_les_routes() {
        for active in Route::ALL {
            let _: iced::Element<'_, ()> = workspace_tabs(active, |route| {
                let _ = route;
            });
            let _: iced::Element<'_, ()> = workspace_tab_controls(active, |route| {
                let _ = route;
            });
        }
    }
}
