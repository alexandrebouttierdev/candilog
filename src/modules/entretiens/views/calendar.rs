//! Grilles du calendrier : mois, semaine et journée, avec leurs panneaux.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::entretiens::components::{event_chip, month_cell, DayState, EventKind};
use crate::modules::entretiens::views::week_start;
use crate::ui::components::button as controls;
use crate::ui::components::icon::Icon;
use crate::ui::components::{badge, inspector, layout, list, state, surface, table, typo};
use crate::ui::format;
use crate::ui::theme::metrics::{size, space};
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use chrono::{Datelike, NaiveDate};
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Nombre d'événements affichés dans une cellule avant repli.
const CELL_EVENTS: usize = 2;

/// Grille mensuelle : six semaines de cellules, légende en pied.
pub fn month_view(app: &App) -> Element<'_, Message> {
    let Some(first) = NaiveDate::from_ymd_opt(app.calendar_year, app.calendar_month, 1) else {
        return state::empty("Mois invalide", "Revenez au mois courant pour continuer.");
    };

    let mut weekdays = row![].spacing(0).width(Length::Fill);
    for index in 0..7 {
        weekdays = weekdays.push(
            container(typo::label(format::weekday_abbrev(index)))
                .width(Length::FillPortion(1))
                .align_x(Alignment::Center),
        );
    }

    let offset = i64::from(first.weekday().num_days_from_monday());
    let mut grid = column![].spacing(0).height(Length::Fill);
    for week in 0_i64..6 {
        let mut line = row![].spacing(0).height(Length::Fill);
        for weekday in 0_i64..7 {
            let date = first + chrono::Duration::days(week * 7 + weekday - offset);
            line = line.push(
                container(cell(app, date))
                    .width(Length::FillPortion(1))
                    .height(Length::Fill),
            );
        }
        grid = grid.push(line);
    }

    surface::panel_bare(
        column![
            container(weekdays)
                .height(size::TABLE_HEADER)
                .padding([0.0, 0.0])
                .align_y(Alignment::Center)
                .style(styles::sunken),
            grid,
            container(legend())
                .padding([space::SM, space::MD])
                .width(Length::Fill),
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Légende des couleurs de la grille : entretiens en vert, relances en ambre.
fn legend<'a>() -> Element<'a, Message> {
    row![
        row![
            badge::marker(Tone::Success, crate::ui::theme::Marker::Solid),
            typo::caption("Entretiens"),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
        row![
            badge::marker(Tone::Warning, crate::ui::theme::Marker::Solid),
            typo::caption("Relances"),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
    ]
    .spacing(space::LG)
    .align_y(Alignment::Center)
    .into()
}

fn cell(app: &App, date: NaiveDate) -> Element<'_, Message> {
    let prefix = date.format("%Y-%m-%d").to_string();
    let interviews: Vec<_> = app
        .data
        .entretiens
        .iter()
        .filter(|event| event.date_entretien.starts_with(&prefix))
        .collect();
    let reminders: Vec<_> = app
        .data
        .relances
        .iter()
        .filter(|event| event.date_relance.starts_with(&prefix))
        .collect();

    let total = interviews.len() + reminders.len();
    let mut events: Vec<Element<'_, Message>> = Vec::new();
    for interview in interviews.iter().take(CELL_EVENTS) {
        events.push(event_chip(
            EventKind::Interview,
            &format::time_part(&interview.date_entretien),
            format::truncate(&interview.type_entretien.to_string(), 12),
        ));
    }
    for reminder in reminders
        .iter()
        .take(CELL_EVENTS.saturating_sub(events.len()))
    {
        events.push(event_chip(
            EventKind::Reminder,
            "",
            format::truncate(&reminder.type_relance, 12),
        ));
    }

    month_cell(
        DayState::new(
            date,
            app.calendar_month,
            chrono::Local::now().date_naive(),
            app.calendar_date,
        ),
        events,
        total.saturating_sub(CELL_EVENTS),
        Message::CalendarDateSelected(date),
    )
}

/// Grille hebdomadaire : sept colonnes d'événements.
pub fn week_view(app: &App) -> Element<'_, Message> {
    let monday = week_start(app.calendar_date);
    let mut week = row![].spacing(0).height(Length::Fill);
    for offset in 0_i64..7 {
        let date = monday + chrono::Duration::days(offset);
        let prefix = date.format("%Y-%m-%d").to_string();
        let mut day = column![container(
            row![
                typo::label(format::weekday_abbrev(
                    date.weekday().num_days_from_monday()
                )),
                layout::spacer(),
                typo::caption(date.format("%d-%m").to_string()),
            ]
            .align_y(Alignment::Center),
        )
        .height(size::TABLE_HEADER)
        .padding([0.0, space::MD])
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .style(styles::sunken)]
        .spacing(0)
        .height(Length::Fill);

        let mut events = column![].spacing(space::XS).padding(space::MD);
        let mut count = 0_usize;
        for interview in app
            .data
            .entretiens
            .iter()
            .filter(|item| item.date_entretien.starts_with(&prefix))
        {
            count += 1;
            events = events.push(event_chip(
                EventKind::Interview,
                &format::time_part(&interview.date_entretien),
                format::truncate(&interview.type_entretien.to_string(), 14),
            ));
        }
        for reminder in app
            .data
            .relances
            .iter()
            .filter(|item| item.date_relance.starts_with(&prefix))
        {
            count += 1;
            events = events.push(event_chip(
                EventKind::Reminder,
                "",
                format::truncate(&reminder.type_relance, 14),
            ));
        }

        let body: Element<'_, Message> = if count == 0 {
            state::empty_slot("—")
        } else {
            surface::scroll(events).height(Length::Fill).into()
        };
        day = day.push(body);

        let is_weekend = date.weekday().num_days_from_monday() >= 5;
        week = week.push(
            container(day)
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .style(if is_weekend {
                    styles::sunken_flat
                } else {
                    styles::panel_flat
                }),
        );
    }
    container(week)
        .height(Length::Fill)
        .style(styles::glass_card)
        .into()
}

/// Agenda d'une journée, avec les comptes rendus et analyses à droite.
pub fn day_view(app: &App) -> Element<'_, Message> {
    let date = app.calendar_date;
    let prefix = date.format("%Y-%m-%d").to_string();

    let mut agenda = column![].spacing(0);
    let mut count = 0_usize;
    for interview in app
        .data
        .entretiens
        .iter()
        .filter(|item| item.date_entretien.starts_with(&prefix))
    {
        count += 1;
        agenda =
            agenda.push(list::row_static(
                badge::badge(format::time_part(&interview.date_entretien), Tone::Success),
                column![
                    typo::item(format!("Entretien · {}", interview.type_entretien)),
                    typo::caption(format::or_dash(interview.lieu.as_deref())),
                ]
                .spacing(0),
                row![
                    controls::ghost(
                        if interview.analyse_ia.is_some() {
                            "Réanalyser"
                        } else {
                            "Analyser"
                        },
                        Some(Icon::Sparkles),
                    )
                    .on_press(Message::AnalyzeInterview(interview.id)),
                    controls::icon_action(
                        Icon::Edit,
                        "Modifier",
                        Message::EditEntretien(interview.id),
                    ),
                    controls::icon_danger(
                        Icon::Trash,
                        "Supprimer",
                        Message::OpenDialog(Dialog::DeleteEntretien(interview.id)),
                    ),
                ]
                .spacing(space::XXS)
                .align_y(Alignment::Center),
            ));
    }
    for reminder in app
        .data
        .relances
        .iter()
        .filter(|item| item.date_relance.starts_with(&prefix))
    {
        count += 1;
        agenda = agenda.push(list::row_static(
            badge::badge("Relance", Tone::Warning),
            column![
                typo::item(reminder.type_relance.clone()),
                typo::caption(format::or_dash(reminder.notes.as_deref())),
            ]
            .spacing(0),
            row![
                controls::icon_action(Icon::Edit, "Modifier", Message::EditRelance(reminder.id)),
                controls::icon_danger(
                    Icon::Trash,
                    "Supprimer",
                    Message::OpenDialog(Dialog::DeleteRelance(reminder.id)),
                ),
            ]
            .spacing(space::XXS)
            .align_y(Alignment::Center),
        ));
    }

    let body: Element<'_, Message> = if count == 0 {
        state::empty(
            "Journée libre",
            "Aucun entretien ni relance planifié à cette date.",
        )
    } else {
        surface::scroll(agenda).height(Length::Fill).into()
    };

    let analyses = analyses_panel(app, &prefix);

    layout::columns([
        surface::panel_bare(
            column![
                table::header(
                    app.layout(),
                    &[
                        table::Column::text("HEURE", 1),
                        table::Column::text("ÉVÉNEMENT", 4),
                        table::Column::trailing("", 130.0),
                    ],
                ),
                body,
            ]
            .height(Length::Fill),
        )
        .width(Length::FillPortion(3))
        .height(Length::Fill)
        .into(),
        analyses,
    ])
}

fn analyses_panel<'a>(app: &'a App, prefix: &str) -> Element<'a, Message> {
    let mut notes = column![].spacing(space::LG);
    let mut count = 0_usize;
    for interview in app
        .data
        .entretiens
        .iter()
        .filter(|item| item.date_entretien.starts_with(prefix))
    {
        count += 1;
        notes = notes.push(inspector::note(
            "Compte rendu",
            interview.compte_rendu.clone(),
        ));
        if let Some(analysis) = &interview.analyse_ia {
            notes = notes.push(inspector::note("Analyse IA", Some(analysis.resume.clone())));
        }
    }
    let body: Element<'a, Message> = if count == 0 {
        state::empty_slot("Sélectionnez une journée avec des entretiens.")
    } else {
        surface::scroll(container(notes).padding(space::XL))
            .height(Length::Fill)
            .into()
    };
    surface::panel_bare(
        column![
            container(surface::section_header(
                "Comptes rendus",
                iced::widget::Space::with_width(0),
            ))
            .padding([0.0, space::XL]),
            surface::divider(),
            body,
        ]
        .height(Length::Fill),
    )
    .width(Length::FillPortion(2))
    .height(Length::Fill)
    .into()
}
