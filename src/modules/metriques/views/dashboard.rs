//! Tableau de bord : les cartes métriques d'abord, les événements et
//! l'activité ensuite, les candidatures récentes en pied de page.
//!
//! La hiérarchie est volontairement inégale : les quatre cartes donnent la
//! tendance, les panneaux « Prochains événements » et « Activité récente »
//! racontent la semaine, le tableau des candidatures récentes ramène au
//! pipeline.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::metriques::components::PipelineCounts;
use crate::navigation::Route;
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::Icon;
use crate::ui::components::layout;
use crate::ui::components::stat_card;
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::Tone;
use iced::widget::{column, row};
use iced::{Alignment, Element, Length};

pub mod panels;

use panels::{activity_panel, recent_panel, upcoming_panel};

/// Rend le tableau de bord.
pub fn view(app: &App) -> Element<'_, Message> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let counts = PipelineCounts::from_stats(&app.data.candidature_stats);
    let due = app.due_reminders(&today);
    let upcoming = app.upcoming_interviews(&today);

    let date_label = format::long_date(chrono::Local::now().date_naive());

    let actions = row![
        controls::ghost("Calendrier", Some(Icon::Calendar))
            .on_press(Message::Navigate(Route::Calendrier)),
        controls::primary("Nouvelle candidature", Some(Icon::Plus))
            .on_press(Message::OpenDialog(Dialog::Candidature)),
    ]
    .spacing(space::SM)
    .align_y(Alignment::Center)
    .into();

    let body = layout::workspace(
        column![
            metric_cards(app, &counts, upcoming, due),
            layout::columns([
                upcoming_panel(app, &today, upcoming)
                    .width(Length::FillPortion(3))
                    .into(),
                activity_panel(app).width(Length::FillPortion(2)).into(),
            ]),
            recent_panel(app),
        ]
        .spacing(space::LG)
        .height(Length::Fill),
    );

    layout::screen(
        header::page_header(Icon::Dashboard, "Tableau de bord", date_label, actions),
        body,
    )
}

/// Les quatre cartes métriques : une ligne quand la fenêtre le permet, deux
/// rangées de deux cartes en dessous du seuil du tableau de bord.
fn metric_cards<'a>(
    app: &App,
    counts: &PipelineCounts,
    upcoming: usize,
    due: usize,
) -> Element<'a, Message> {
    let [active, interviews, rate, reminders] = [
        stat_card::metric_icon_tinted(
            "Candidatures actives",
            counts.active().to_string(),
            Tone::Accent,
            Icon::Applications,
        ),
        stat_card::metric_icon_tinted(
            "Entretiens à venir",
            upcoming.to_string(),
            Tone::Accent,
            Icon::Calendar,
        ),
        stat_card::metric_icon_tinted(
            "Taux de réponse",
            format!("{} %", counts.response_rate()),
            Tone::Success,
            Icon::Chart,
        ),
        stat_card::metric_icon_tinted(
            "Relances à traiter",
            due.to_string(),
            Tone::Warning,
            Icon::Alert,
        ),
    ];

    if app.layout().dashboard_two_columns() {
        row![active, interviews, rate, reminders]
            .spacing(space::MD)
            .into()
    } else {
        column![
            row![active, interviews].spacing(space::MD),
            row![rate, reminders].spacing(space::MD),
        ]
        .spacing(space::MD)
        .into()
    }
}
