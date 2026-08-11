//! Barre latérale de navigation : 5 sections, item actif sur fond secondaire.

use crate::navigation::{Route, Section};
use crate::ui::components::icon::{self, Ink};
use crate::ui::components::typo;
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use iced::widget::{button, column, container, row, scrollable};
use iced::{Alignment, Border, Color, Element, Length, Theme};
use std::collections::BTreeMap;

/// Routes regroupées par section, dans l'ordre de la barre latérale.
#[must_use]
pub fn rows_by_section(active: Route) -> BTreeMap<Section, Vec<(Route, bool)>> {
    let mut groups = BTreeMap::new();
    for route in Route::ALL {
        groups
            .entry(route.section())
            .or_insert_with(Vec::new)
            .push((route, route == active));
    }
    groups
}

/// Barre latérale complète (216 px), pied `footer` épinglé.
pub fn sidebar<'a, Message: Clone + 'a>(
    active: Route,
    on_navigate: impl Fn(Route) -> Message + 'a,
    footer: Element<'a, Message>,
) -> Element<'a, Message> {
    let rows = rows_by_section(active);
    let mut body = column![]
        .spacing(space::XXS)
        .padding([space::SM, space::MD]);
    for section in Section::ALL {
        if let Some(label) = section.label() {
            body = body.push(
                container(typo::text_uppercase(label, font::CAPTION, font::SEMIBOLD)).padding(
                    iced::Padding::new(space::MD)
                        .bottom(space::XS)
                        .right(space::SM),
                ),
            );
        }
        for (route, is_active) in rows.get(&section).unwrap_or(&Vec::new()) {
            let control = button(crate::ui::components::button::vcenter(
                row![
                    icon::icon(
                        route.icon(),
                        17.0,
                        if *is_active { Ink::Accent } else { Ink::Muted },
                    ),
                    typo::body(route.label()),
                ]
                .spacing(9.0)
                .align_y(Alignment::Center),
            ))
            .width(Length::Fill)
            .height(size::ACTION + 4.0)
            .padding([0.0, space::SM])
            .style(styles::nav_item(*is_active))
            .on_press(on_navigate(*route));
            body = body.push(control);
        }
    }

    container(
        column![
            scrollable(body).width(Length::Fill).height(Length::Fill),
            container(
                row![iced::widget::Space::with_width(Length::Fill), footer]
                    .align_y(Alignment::Center),
            )
            .padding([space::SM, space::MD])
            .style(move |theme: &Theme| {
                let palette = tokens(theme);
                container::Style {
                    border: Border {
                        color: Color {
                            a: 0.60,
                            ..palette.border
                        },
                        width: stroke::HAIRLINE,
                        radius: radius::NONE.into(),
                    },
                    ..container::Style::default()
                }
            }),
        ]
        .height(Length::Fill),
    )
    .width(size::SIDEBAR)
    .height(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::{rows_by_section, sidebar};
    use crate::navigation::Route;

    #[test]
    fn la_sidebar_s_instancie() {
        let _: iced::Element<'_, ()> = sidebar(
            Route::Dashboard,
            |route| {
                let _ = route;
            },
            iced::widget::Space::with_width(0).into(),
        );
    }

    #[test]
    fn chaque_route_apparait_dans_une_seule_section() {
        let groups = rows_by_section(Route::Dashboard);
        let total: usize = groups.values().map(Vec::len).sum();
        assert_eq!(total, Route::ALL.len());
    }
}
