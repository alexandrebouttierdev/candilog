//! Rendu des objets d'agenda : cellules de calendrier et lignes d'événement.

use crate::ui::components::{badge, typo};
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::styles::mix_panel;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::{Marker, Tone};
use chrono::{Datelike, NaiveDate};
use iced::widget::{button, column, container, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

pub mod form;

/// Nature d'un événement d'agenda.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// Entretien planifié.
    Interview,
    /// Relance à effectuer.
    Reminder,
}

impl EventKind {
    /// Ton sémantique de l'événement.
    #[must_use]
    pub const fn tone(self) -> Tone {
        match self {
            Self::Interview => Tone::Success,
            Self::Reminder => Tone::Warning,
        }
    }

    /// Libellé court affiché dans une pastille.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Interview => "Entretien",
            Self::Reminder => "Relance",
        }
    }
}

/// Événement compact affiché dans une cellule de calendrier : fond teinté du
/// ton de la nature d'événement, texte 11 px, rayon 6 px.
pub fn event_chip<'a, Message: 'a>(
    kind: EventKind,
    time: &str,
    title: String,
) -> Element<'a, Message> {
    let mut line = row![badge::marker(kind.tone(), Marker::Solid)]
        .spacing(space::SM - 1.0)
        .align_y(Alignment::Center);
    if !time.is_empty() {
        line = line.push(typo::caption(time.to_owned()));
    }
    line = line.push(typo::caption(title));
    container(line)
        .height(size::TAG)
        .width(Length::Fill)
        .align_y(Alignment::Center)
        .padding([0.0, space::XS])
        .style(move |theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(kind.tone().surface(&palette))),
                border: Border {
                    radius: radius::CONTROL.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Qualification d'un jour de la grille mensuelle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayState {
    /// Date rendue par la cellule.
    pub date: NaiveDate,
    /// Le jour appartient au mois affiché.
    pub in_month: bool,
    /// Le jour est la date du jour.
    pub is_today: bool,
    /// Le jour est un samedi ou un dimanche.
    pub is_weekend: bool,
    /// Le jour porte la sélection courante.
    pub selected: bool,
}

impl DayState {
    /// Qualifie une date par rapport au mois affiché et à la sélection.
    #[must_use]
    pub fn new(date: NaiveDate, month: u32, today: NaiveDate, selected: NaiveDate) -> Self {
        Self {
            date,
            in_month: date.month() == month,
            is_today: date == today,
            is_weekend: date.weekday().num_days_from_monday() >= 5,
            selected: date == selected,
        }
    }
}

/// Cellule d'une grille mensuelle.
pub fn month_cell<'a, Message: Clone + 'a>(
    day_state: DayState,
    events: Vec<Element<'a, Message>>,
    overflow: usize,
    on_select: Message,
) -> Element<'a, Message> {
    let DayState {
        date,
        in_month,
        is_today,
        is_weekend,
        selected,
    } = day_state;
    let day: Element<'a, Message> = if is_today {
        badge::today_chip(date.day().to_string())
    } else {
        container(typo::caption(date.day().to_string()))
            .center_x(Length::Fixed(18.0))
            .center_y(Length::Fixed(18.0))
            .into()
    };

    let mut content =
        column![row![day, Space::with_width(Length::Fill)].align_y(Alignment::Center)]
            .spacing(2)
            .width(Length::Fill);
    for event in events {
        content = content.push(event);
    }
    if overflow > 0 {
        content = content.push(
            container(typo::caption(format!("+{overflow}")))
                .width(Length::Fill)
                .align_x(Alignment::Center),
        );
    }

    button(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(space::SM)
        .style(move |theme: &Theme, status| {
            let palette = tokens(theme);
            let glass = mix_panel(palette.panel, palette.canvas, 0.55);
            let background = if selected && !is_today {
                palette.selection
            } else if matches!(status, button::Status::Hovered) {
                palette.hover
            } else if !in_month {
                palette.sunken
            } else if is_today {
                alpha(palette.selection, 0.08)
            } else if is_weekend {
                mix_panel(palette.sunken, palette.canvas, 0.55)
            } else {
                glass
            };
            button::Style {
                background: Some(Background::Color(background)),
                text_color: if in_month {
                    palette.text
                } else {
                    palette.text_muted
                },
                border: Border {
                    color: palette.border,
                    width: stroke::HAIRLINE,
                    radius: 0.0.into(),
                },
                ..button::Style::default()
            }
        })
        .on_press(on_select)
        .into()
}

#[cfg(test)]
#[path = "tests/mod/mod.rs"]
mod tests;
