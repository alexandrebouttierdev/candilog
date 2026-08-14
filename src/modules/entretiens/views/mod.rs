//! Écran Calendrier : grille mensuelle, semaine et journée.

use crate::app::message::CalendarView;
use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::entretiens::model::Entretien;
use crate::modules::relances::model::Relance;
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::Icon;
use crate::ui::components::sidebar::workspace_tab_controls;
use crate::ui::components::stat_card;
use crate::ui::components::{layout, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use chrono::{Datelike, NaiveDate};

pub mod calendar;

use calendar::{day_view, month_view, week_view};
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

fn period_label(app: &App) -> String {
    match app.calendar_view {
        CalendarView::Month => format!(
            "{} {}",
            format::month_name(app.calendar_month),
            app.calendar_year
        ),
        CalendarView::Week => {
            let monday = week_start(app.calendar_date);
            let sunday = monday + chrono::Duration::days(6);
            format!("{} – {}", monday.format("%d-%m"), sunday.format("%d-%m-%Y"))
        }
        CalendarView::Day => format!(
            "{} {} {}",
            app.calendar_date.day(),
            format::month_name(app.calendar_date.month()),
            app.calendar_date.year()
        ),
    }
}

pub(crate) fn week_start(date: NaiveDate) -> NaiveDate {
    date - chrono::Duration::days(i64::from(date.weekday().num_days_from_monday()))
}

/// Rend l'écran du calendrier.
pub fn view(app: &App) -> Element<'_, Message> {
    let actions = row![
        controls::ghost("Relance", Some(Icon::Plus)).on_press(Message::OpenDialog(Dialog::Relance)),
        controls::primary("Entretien", Some(Icon::Plus))
            .on_press(Message::OpenDialog(Dialog::Entretien)),
    ]
    .spacing(space::SM)
    .align_y(Alignment::Center)
    .into();

    let calendar = match app.calendar_view {
        CalendarView::Month => month_view(app),
        CalendarView::Week => week_view(app),
        CalendarView::Day => day_view(app),
    };

    layout::screen(
        header::workspace_header(
            Icon::Calendar,
            "Calendrier et échéances",
            workspace_tab_controls(app.route, Message::Navigate),
            actions,
        ),
        layout::workspace(
            column![nav_band(app), metrics_row(app), calendar]
                .spacing(space::LG)
                .height(Length::Fill),
        ),
    )
}

/// Barre de navigation : période précédente/suivante, aujourd'hui,
/// libellé de période et granularité du calendrier.
fn nav_band(app: &App) -> Element<'_, Message> {
    container(
        row![
            controls::icon_action(
                Icon::ArrowLeft,
                "Période précédente",
                Message::PreviousMonth,
            ),
            controls::icon_action(Icon::ArrowRight, "Période suivante", Message::NextMonth),
            controls::ghost("Aujourd'hui", None).on_press(Message::CurrentMonth),
            typo::item_strong(period_label(app)),
            layout::spacer(),
            controls::segmented([
                controls::segment("Mois", app.calendar_view == CalendarView::Month)
                    .on_press(Message::CalendarViewChanged(CalendarView::Month)),
                controls::segment("Semaine", app.calendar_view == CalendarView::Week)
                    .on_press(Message::CalendarViewChanged(CalendarView::Week)),
                controls::segment("Jour", app.calendar_view == CalendarView::Day)
                    .on_press(Message::CalendarViewChanged(CalendarView::Day)),
            ]),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
    )
    .padding([space::SM, space::LG])
    .width(Length::Fill)
    .style(styles::form_group)
    .into()
}

/// Les trois métriques du mois affiché : événements, entretiens, relances.
fn metrics_row(app: &App) -> Element<'_, Message> {
    let (total, interviews, reminders) = month_counts(
        &app.data.entretiens,
        &app.data.relances,
        app.calendar_year,
        app.calendar_month,
    );
    row![
        stat_card::metric_icon_tinted(
            "Événements",
            total.to_string(),
            Tone::Neutral,
            Icon::Calendar,
        ),
        stat_card::metric_icon_tinted(
            "Entretiens",
            interviews.to_string(),
            Tone::Success,
            Icon::Calendar,
        ),
        stat_card::metric_icon_tinted(
            "Relances",
            reminders.to_string(),
            Tone::Warning,
            Icon::Alert,
        ),
    ]
    .spacing(space::MD)
    .into()
}

/// Comptes du mois affiché : événements totaux, entretiens et relances.
///
/// La correspondance se fait sur le préfixe `AAAA-MM` : les lignes reprises
/// de l'ancienne base portent un horodatage complet qui commence par la date.
fn month_counts(
    entretiens: &[Entretien],
    relances: &[Relance],
    year: i32,
    month: u32,
) -> (usize, usize, usize) {
    let prefix = format!("{year:04}-{month:02}");
    let interviews = entretiens
        .iter()
        .filter(|item| item.date_entretien.starts_with(&prefix))
        .count();
    let reminders = relances
        .iter()
        .filter(|item| item.date_relance.starts_with(&prefix))
        .count();
    (interviews + reminders, interviews, reminders)
}

#[cfg(test)]
#[path = "tests/mod/mod.rs"]
mod tests;
