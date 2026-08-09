//! Rendu des objets d'agenda : cellules de calendrier et lignes d'événement.

use crate::ui::components::{badge, typo};
use crate::ui::theme::metrics::{space, stroke};
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::{Marker, Tone};
use chrono::{Datelike, NaiveDate};
use iced::widget::{button, column, container, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

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

/// Événement compact affiché dans une cellule de calendrier.
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
        .height(16)
        .width(Length::Fill)
        .align_y(Alignment::Center)
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
        content = content.push(typo::caption(format!("+{overflow}")));
    }

    button(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(space::SM)
        .style(move |theme: &Theme, status| {
            let palette = tokens(theme);
            let background = if selected && !is_today {
                palette.selection
            } else if matches!(status, button::Status::Hovered) {
                palette.hover
            } else if !in_month {
                palette.canvas
            } else if is_weekend {
                palette.sunken
            } else {
                palette.panel
            };
            button::Style {
                background: Some(Background::Color(background)),
                text_color: if in_month {
                    palette.text
                } else {
                    palette.text_muted
                },
                border: Border {
                    color: if is_today {
                        palette.accent
                    } else {
                        palette.border
                    },
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
mod tests {
    use super::EventKind;
    use crate::ui::theme::Tone;

    #[test]
    fn les_deux_natures_d_evenement_sont_distinguables() {
        assert_ne!(EventKind::Interview.tone(), EventKind::Reminder.tone());
        assert_ne!(EventKind::Interview.label(), EventKind::Reminder.label());
        assert_eq!(EventKind::Interview.tone(), Tone::Success);
        assert_eq!(EventKind::Reminder.tone(), Tone::Warning);
    }
}
