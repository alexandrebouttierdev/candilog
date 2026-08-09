//! Tableau de bord : ce qui demande une action d'abord, le reste ensuite.
//!
//! La hiérarchie est volontairement inégale. Une colonne « à traiter » guide
//! le regard ; les indicateurs de contexte restent en retrait.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::candidatures::components::status_badge;
use crate::modules::metriques::components::PipelineCounts;
use crate::navigation::Route;
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon};
use crate::ui::components::{layout, list, meter, state, surface, toolbar, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Rend le tableau de bord.
pub fn view(app: &App) -> Element<'_, Message> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let counts = PipelineCounts::from_candidates(&app.data.candidatures);
    let due = app.due_reminders(&today);
    let upcoming = app.upcoming_interviews(&today);

    let trailing = toolbar::group([
        controls::ghost("Calendrier", Some(Icon::Calendar))
            .on_press(Message::Navigate(Route::Calendrier))
            .into(),
        controls::primary("Nouvelle candidature", Some(Icon::Plus))
            .on_press(Message::OpenDialog(Dialog::Candidature))
            .into(),
    ]);

    let body = layout::workspace(
        column![
            indicators(counts, upcoming, due),
            layout::columns([attention_panel(app, &today, due, counts), recent_panel(app),]),
        ]
        .spacing(space::LG)
        .height(Length::Fill),
    );

    layout::screen(
        toolbar::toolbar(
            "Tableau de bord",
            typo::meta(format::long_date(chrono::Local::now().date_naive())),
            trailing,
        ),
        body,
    )
}

fn indicators<'a>(counts: PipelineCounts, upcoming: usize, due: usize) -> Element<'a, Message> {
    surface::panel(
        row![
            meter::metric(
                "Candidatures actives",
                counts.active().to_string(),
                Tone::Accent
            ),
            layout::spacer(),
            meter::metric(
                "Entretiens à venir",
                upcoming.to_string(),
                if upcoming > 0 {
                    Tone::Success
                } else {
                    Tone::Neutral
                },
            ),
            layout::spacer(),
            meter::metric(
                "Taux de réponse",
                format!("{} %", counts.response_rate()),
                Tone::Neutral,
            ),
            layout::spacer(),
            if due > 0 {
                meter::metric_with_hint("Relances", due.to_string(), "à traiter", Tone::Warning)
            } else {
                meter::metric("Relances", "0", Tone::Neutral)
            },
        ]
        .spacing(space::MAX)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .into()
}

fn attention_panel<'a>(
    app: &'a App,
    today: &str,
    due: usize,
    counts: PipelineCounts,
) -> Element<'a, Message> {
    let mut items = column![].spacing(0);
    let mut count = 0_usize;

    for reminder in app
        .data
        .relances
        .iter()
        .filter(|item| item.date_relance.as_str() <= today)
        .take(4)
    {
        count += 1;
        let poste = app
            .data
            .candidatures
            .iter()
            .find(|item| item.id == reminder.candidature_id)
            .map_or_else(|| "Candidature".to_owned(), |item| item.poste.clone());
        items = items.push(list::row_static(
            icon::toned(Icon::Alert, Tone::Warning),
            column![
                typo::body(format::truncate(&poste, 38)),
                typo::caption(format!(
                    "Relance {} · {}",
                    reminder.type_relance,
                    format::compact_date(&reminder.date_relance)
                )),
            ]
            .spacing(0),
            controls::ghost("Ouvrir", None).on_press(Message::Navigate(Route::Calendrier)),
        ));
    }

    for interview in app
        .data
        .entretiens
        .iter()
        .filter(|item| item.date_entretien.as_str() >= today)
        .take(4)
    {
        count += 1;
        items = items.push(list::row_static(
            icon::toned(Icon::Calendar, Tone::Success),
            column![
                typo::body(interview.type_entretien.to_string()),
                typo::caption(format::compact_datetime(&interview.date_entretien)),
            ]
            .spacing(0),
            typo::caption(format::or_dash(interview.lieu.as_deref())),
        ));
    }

    if counts.pending > 0 {
        count += 1;
        items = items.push(list::row_static(
            icon::muted(Icon::Inbox),
            column![
                typo::body("Candidatures sans réponse"),
                typo::caption(format::plural(counts.pending, "dossier", "dossiers")),
            ]
            .spacing(0),
            controls::ghost("Voir", None).on_press(Message::Navigate(Route::Candidatures)),
        ));
    }

    let body: Element<'a, Message> = if count == 0 {
        state::empty(
            "Rien à traiter",
            "Aucune relance en retard ni entretien planifié.",
        )
    } else {
        surface::scroll(items).height(Length::Fill).into()
    };

    surface::panel_bare(
        column![
            container(surface::section_header(
                "À traiter",
                crate::ui::components::badge::count_toned(due, Tone::Warning),
            ))
            .padding([0.0, space::XL]),
            surface::divider(),
            body,
        ]
        .height(Length::Fill),
    )
    .width(Length::FillPortion(3))
    .height(Length::Fill)
    .into()
}

fn recent_panel(app: &App) -> Element<'_, Message> {
    let mut items = column![].spacing(0);
    let mut recent: Vec<_> = app.data.candidatures.iter().collect();
    recent.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

    for candidate in recent.iter().take(8) {
        items = items.push(list::row_static(
            crate::modules::candidatures::components::glyph(candidate.statut),
            column![
                typo::body(format::truncate(&candidate.poste, 34)),
                typo::caption(format::or_else(
                    candidate.entreprise_nom.as_deref(),
                    "Entreprise inconnue"
                )),
            ]
            .spacing(0),
            status_badge(candidate.statut),
        ));
    }

    let body: Element<'_, Message> = if recent.is_empty() {
        state::empty_with_action(
            "Aucune candidature",
            "Ajoutez votre première candidature pour démarrer le suivi.",
            "Nouvelle candidature",
            Message::OpenDialog(Dialog::Candidature),
        )
    } else {
        surface::scroll(items).height(Length::Fill).into()
    };

    surface::panel_bare(
        column![
            container(surface::section_header(
                "Activité récente",
                controls::ghost("Tout voir", Some(Icon::ChevronRight))
                    .on_press(Message::Navigate(Route::Candidatures)),
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
