//! Panneaux d'activité et d'événements du tableau de bord.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::candidatures::components::{contract_short, status_badge};
use crate::navigation::Route;
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::table::{self, Column};
use crate::ui::components::{badge, bar, list, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{column, container, Container};
use iced::{Element, Length};

const RECENT_COLUMNS: [Column; 4] = [
    Column::text("POSTE", 4),
    Column::text("ENTREPRISE", 3).secondary(),
    Column::centered("STATUT", 126.0).secondary(),
    Column::trailing("MISE À JOUR", 104.0).secondary(),
];

/// Panneau « Prochains événements » : entretiens à venir puis relances
/// échues, chacun portant une pastille d'icône teintée.
pub(super) fn upcoming_panel<'a>(
    app: &'a App,
    today: &str,
    upcoming: usize,
) -> Container<'a, Message> {
    let mut rows = column![].spacing(0);
    let mut count = 0_usize;

    let mut interviews: Vec<_> = app
        .data
        .entretiens
        .iter()
        .filter(|item| item.date_entretien.as_str() >= today)
        .collect();
    interviews.sort_by(|left, right| left.date_entretien.cmp(&right.date_entretien));

    for interview in interviews.into_iter().take(4) {
        count += 1;
        rows = rows.push(list::row_static(
            event_icon(Icon::Calendar, Tone::Success),
            column![
                typo::item(interview.type_entretien.to_string()),
                typo::text_mono(
                    format::compact_datetime(&interview.date_entretien),
                    11.0,
                    font::MONO_REGULAR,
                ),
            ]
            .spacing(space::XXS),
            typo::caption(format::or_dash(interview.lieu.as_deref())),
        ));
    }

    let mut reminders: Vec<_> = app
        .data
        .relances
        .iter()
        .filter(|item| item.date_relance.as_str() <= today)
        .collect();
    reminders.sort_by(|left, right| left.date_relance.cmp(&right.date_relance));

    for reminder in reminders.into_iter().take(4) {
        count += 1;
        let poste = app
            .data
            .candidatures
            .iter()
            .find(|item| item.id == reminder.candidature_id)
            .map_or_else(|| "Candidature".to_owned(), |item| item.poste.clone());
        rows = rows.push(list::row_static(
            event_icon(Icon::Alert, Tone::Warning),
            column![
                typo::item(format::truncate(&poste, 30)),
                typo::text_mono(
                    format::compact_date(&reminder.date_relance),
                    11.0,
                    font::MONO_REGULAR,
                ),
            ]
            .spacing(space::XXS),
            typo::caption(reminder.type_relance.clone()),
        ));
    }

    let body: Element<'a, Message> = if count == 0 {
        state::empty("Rien à venir", "Aucun entretien ni relance planifiés.")
    } else {
        surface::scroll(rows).height(Length::Fill).into()
    };

    surface::panel(
        column![
            surface::section_header("Prochains événements", badge::count(upcoming)),
            surface::divider(),
            body,
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
}

/// Panneau « Activité récente » : les envois des sept derniers jours en
/// barres, avec la valeur et le jour superposés à chaque barre.
pub(super) fn activity_panel(app: &App) -> Container<'_, Message> {
    let days = last_seven_days(&app.data.candidature_stats.activity_by_day);
    let maximum = days
        .iter()
        .map(|(_, count)| *count)
        .max()
        .unwrap_or(1)
        .max(1);
    let mut chart = column![].spacing(space::SM).width(Length::Fill);
    for (date, count) in &days {
        chart = chart.push(bar::barre(
            format::compact_date(&date.format("%Y-%m-%d").to_string()),
            format::plural(*count, "envoi", "envois"),
            (*count as f32 / maximum as f32) * 100.0,
            Tone::Accent,
        ));
    }

    surface::panel(
        column![
            surface::section_header(
                "Activité récente",
                badge::badge("7 derniers jours", Tone::Neutral),
            ),
            surface::divider(),
            container(chart).padding([space::MD, 0.0]),
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
}

/// Panneau « Candidatures récentes » : les cinq plus récentes mises à jour,
/// avec le statut et la date de chaque candidature.
pub(super) fn recent_panel(app: &App) -> Container<'_, Message> {
    let mut items = column![];
    let mut recent: Vec<_> = app.data.candidatures.iter().collect();
    recent.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));

    for candidate in recent.iter().take(5) {
        let cells = vec![
            table::cell(
                RECENT_COLUMNS[0],
                column![
                    typo::item(format::truncate(&candidate.poste, 46)),
                    typo::caption(contract_short(candidate.type_contrat)),
                ]
                .spacing(0),
            ),
            table::cell(
                RECENT_COLUMNS[1],
                typo::meta(format::truncate(
                    &format::or_else(candidate.entreprise_nom.as_deref(), "Entreprise inconnue"),
                    42,
                )),
            ),
            table::cell(RECENT_COLUMNS[2], status_badge(candidate.statut)),
            table::cell(
                RECENT_COLUMNS[3],
                typo::text_mono(
                    format::compact_date(&candidate.updated_at),
                    11.0,
                    font::MONO_REGULAR,
                ),
            ),
        ];
        items = items.push(table::row_button(
            app.layout(),
            cells,
            app.selected_candidate == Some(candidate.id),
            Message::OpenDialog(Dialog::CandidatureDetail(candidate.id)),
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
                "Candidatures récentes",
                controls::ghost("Tout voir", Some(Icon::ChevronRight))
                    .on_press(Message::Navigate(Route::Candidatures)),
            ))
            .padding([0.0, space::LG]),
            surface::divider(),
            table::header(app.layout(), &RECENT_COLUMNS),
            body,
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
}

/// Pastille d'événement 36 px : icône du ton sur fond teinté du même ton.
fn event_icon<'a, Message: 'a>(glyph: Icon, tone: Tone) -> Element<'a, Message> {
    container(icon::icon(glyph, icon::MD, Ink::Toned(tone)))
        .width(36.0)
        .height(36.0)
        .center(Length::Fixed(36.0))
        .style(styles::toned(tone))
        .into()
}

/// Dates des sept derniers jours avec le compte de candidatures envoyées
/// chaque jour, depuis le jour le plus ancien jusqu'à aujourd'hui.
///
/// La comparaison se fait sur le préfixe `AAAA-MM-JJ` : les lignes reprises
/// de l'ancienne base portent un horodatage complet qui commence par la date.
fn last_seven_days(activity: &[(String, u64)]) -> Vec<(chrono::NaiveDate, usize)> {
    let today = chrono::Local::now().date_naive();
    let mut days = Vec::with_capacity(7);
    let mut cursor = today.checked_sub_days(chrono::Days::new(6));
    while let Some(date) = cursor {
        let prefix = date.format("%Y-%m-%d").to_string();
        let count = activity
            .iter()
            .find(|(date, _)| date == &prefix)
            .map_or(0, |(_, count)| {
                usize::try_from(*count).unwrap_or(usize::MAX)
            });
        days.push((date, count));
        cursor = date.succ_opt().filter(|next| next <= &today);
    }
    days
}

#[cfg(test)]
#[path = "tests/panels/mod.rs"]
mod tests;
