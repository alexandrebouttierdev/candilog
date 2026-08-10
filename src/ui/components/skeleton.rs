//! Squelettes de chargement (états d'attente avant données réelles).

use crate::ui::theme::metrics::{radius, space};
use crate::ui::theme::tokens::tokens;
use iced::widget::{column, container, row, Row, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

/// Bloc de squelette gris, pulsé via un dégradé discret.
pub fn block<'a, Message: 'a>(width: Length, height: f32) -> Element<'a, Message> {
    container(Space::new(width, Length::Fixed(height)))
        .style(|theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(Color {
                    a: if palette.is_dark { 0.08 } else { 0.11 },
                    ..palette.text
                })),
                border: Border {
                    radius: radius::CONTROL.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Nombre de lignes d'un squelette de panneau.
#[must_use]
pub fn skeleton_lines(count: usize) -> Vec<f32> {
    (0..count)
        .map(|index| if index % 5 == 4 { 1.0 } else { 0.0 })
        .collect()
}

/// Squelette d'une carte métrique.
pub fn metric<'a, Message: 'a>() -> Element<'a, Message> {
    column![
        block(Length::FillPortion(2), 12.0),
        block(Length::FillPortion(1), 28.0)
    ]
    .spacing(space::SM)
    .padding(space::LG)
    .into()
}

/// Squelette de panneau avec un titre et `lines` lignes.
pub fn panel<'a, Message: 'a>(lines: usize) -> Element<'a, Message> {
    let mut body = column![
        block(Length::Fixed(120.0), 16.0),
        block(Length::Fixed(180.0), 12.0),
    ]
    .spacing(space::MD);
    for _ in 0..lines {
        body = body.push(block(Length::Fill, 14.0));
    }
    container(body)
        .padding(space::XL)
        .width(Length::Fill)
        .into()
}

/// Ligne d'une semaine du squelette de calendrier : 7 cases égales.
fn calendar_week<'a, Message: 'a>() -> Row<'a, Message> {
    let mut week = row![].spacing(space::XS);
    for _ in 0..7 {
        week = week.push(block(Length::FillPortion(1), 96.0));
    }
    week
}

/// Squelette d'une page complète, variantes du Suspense candilog-desktop.
pub enum PageSkeleton {
    /// Page standard : en-tête + deux panneaux.
    Default,
    /// Tableau de bord : 4 métriques + 2 panneaux.
    Dashboard,
    /// Liste : toolbar + lignes.
    List,
    /// Kanban : 4 colonnes de cartes.
    Board,
    /// Calendrier : grille de 7×5.
    Calendar,
    /// Formulaire : deux colonnes de champs.
    Form,
    /// Cartes en grille.
    Cards,
}

impl PageSkeleton {
    /// Rend le squelette de la page.
    pub fn render<'a, Message: 'a>(&self) -> Element<'a, Message> {
        let header = row![
            block(Length::Fixed(44.0), 44.0),
            column![
                block(Length::Fixed(176.0), 20.0),
                block(Length::Fixed(288.0), 12.0)
            ]
            .spacing(space::XS),
            Space::with_width(Length::Fill),
            block(Length::Fixed(144.0), 36.0),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center);

        let body: Element<'a, Message> = match self {
            Self::Default => column![
                header,
                row![
                    container(panel(6)).width(Length::FillPortion(3)),
                    container(panel(4)).width(Length::FillPortion(2)),
                ]
                .spacing(space::LG),
            ]
            .spacing(space::XL)
            .into(),
            Self::Dashboard => column![
                header,
                row![metric(), metric(), metric(), metric()].spacing(space::MD),
                row![
                    container(panel(6)).width(Length::FillPortion(3)),
                    container(panel(5)).width(Length::FillPortion(2)),
                ]
                .spacing(space::LG),
            ]
            .spacing(space::XL)
            .into(),
            Self::List => column![
                header,
                container(
                    column![row![
                        block(Length::FillPortion(2), 34.0),
                        block(Length::Fixed(160.0), 34.0),
                        Space::with_width(Length::Fill),
                        block(Length::Fixed(120.0), 34.0),
                    ]
                    .spacing(space::MD)]
                    .spacing(space::LG)
                    .padding(space::LG),
                )
                .style(crate::ui::theme::styles::glass_card),
                column![
                    block(Length::Fill, 42.0),
                    block(Length::Fill, 42.0),
                    block(Length::Fill, 42.0)
                ]
                .spacing(space::XS),
            ]
            .spacing(space::LG)
            .into(),
            Self::Board => column![
                header,
                row![
                    column![
                        block(Length::Fill, 24.0),
                        block(Length::Fill, 120.0),
                        block(Length::Fill, 96.0)
                    ]
                    .spacing(space::MD),
                    column![
                        block(Length::Fill, 24.0),
                        block(Length::Fill, 96.0),
                        block(Length::Fill, 140.0)
                    ]
                    .spacing(space::MD),
                    column![
                        block(Length::Fill, 24.0),
                        block(Length::Fill, 140.0),
                        block(Length::Fill, 96.0)
                    ]
                    .spacing(space::MD),
                    column![
                        block(Length::Fill, 24.0),
                        block(Length::Fill, 96.0),
                        block(Length::Fill, 120.0)
                    ]
                    .spacing(space::MD),
                ]
                .spacing(space::LG),
            ]
            .spacing(space::XL)
            .into(),
            Self::Calendar => column![
                header,
                container(
                    column![
                        row![
                            block(Length::FillPortion(1), 20.0),
                            block(Length::FillPortion(1), 20.0),
                            block(Length::FillPortion(1), 20.0),
                        ]
                        .spacing(space::MD),
                        column![
                            calendar_week(),
                            calendar_week(),
                            calendar_week(),
                            calendar_week(),
                            calendar_week(),
                        ]
                        .spacing(space::XS),
                    ]
                    .spacing(space::LG)
                    .padding(space::XL),
                )
                .style(crate::ui::theme::styles::glass_card),
            ]
            .spacing(space::XL)
            .into(),
            Self::Form => column![
                header,
                row![
                    container(panel(8)).width(Length::FillPortion(3)),
                    container(panel(4)).width(Length::FillPortion(2)),
                ]
                .spacing(space::LG),
            ]
            .spacing(space::XL)
            .into(),
            Self::Cards => column![
                header,
                row![metric(), metric(), metric(), metric()].spacing(space::MD),
                row![
                    container(panel(3)).width(Length::FillPortion(1)),
                    container(panel(3)).width(Length::FillPortion(1)),
                    container(panel(3)).width(Length::FillPortion(1)),
                ]
                .spacing(space::LG),
            ]
            .spacing(space::XL)
            .into(),
        };

        column![body]
            .padding([space::PAGE_Y, space::PAGE])
            .spacing(space::XL)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::skeleton_lines;

    #[test]
    fn les_lignes_de_squelette_respectent_leur_nombre() {
        assert_eq!(skeleton_lines(3).len(), 3);
        assert_eq!(skeleton_lines(0).len(), 0);
    }
}
