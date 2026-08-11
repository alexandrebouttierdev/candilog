//! Tableau de bord : les cartes métriques d'abord, les événements et
//! l'activité ensuite, les candidatures récentes en pied de page.
//!
//! La hiérarchie est volontairement inégale : les quatre cartes donnent la
//! tendance, les panneaux « Prochains événements » et « Activité récente »
//! racontent la semaine, le tableau des candidatures récentes ramène au
//! pipeline.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::candidatures::components::{contract_short, status_badge};
use crate::modules::candidatures::model::Candidature;
use crate::modules::metriques::components::PipelineCounts;
use crate::navigation::Route;
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::stat_card;
use crate::ui::components::table::{self, Column};
use crate::ui::components::{badge, layout, list, sparkline, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use chrono::Datelike;
use iced::widget::{column, container, row, Container, Space, Stack};
use iced::{Alignment, Element, Length, Padding};

const RECENT_COLUMNS: [Column; 4] = [
    Column::text("POSTE", 4),
    Column::text("ENTREPRISE", 3).secondary(),
    Column::centered("STATUT", 126.0).secondary(),
    Column::trailing("MISE À JOUR", 104.0).secondary(),
];

/// Rend le tableau de bord.
pub fn view(app: &App) -> Element<'_, Message> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let counts = PipelineCounts::from_candidates(&app.data.candidatures);
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

/// Panneau « Prochains événements » : entretiens à venir puis relances
/// échues, chacun portant une pastille d'icône teintée.
fn upcoming_panel<'a>(app: &'a App, today: &str, upcoming: usize) -> Container<'a, Message> {
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
fn activity_panel(app: &App) -> Container<'_, Message> {
    let days = last_seven_days(&app.data.candidatures);
    let mut counts = [0_usize; 7];
    for (slot, (_, count)) in counts.iter_mut().zip(&days) {
        *slot = *count;
    }
    let heights = sparkline::bar_heights(counts);

    let mut overlay = row![].width(Length::Fill).height(Length::Fill);
    for (date, count) in &days {
        overlay = overlay.push(
            column![
                typo::text_mono(count.to_string(), 11.0, font::MONO_REGULAR),
                Space::with_height(Length::Fill),
                typo::caption(day_letter(date.weekday().num_days_from_monday())),
            ]
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .align_x(Alignment::Center),
        );
    }

    // Un seul canvas dessine les barres ; une grille de textes par-dessus lui
    // ajoute la valeur (en haut) et le jour (en bas). Le canvas réserve le
    // bas de sa zone pour que les barres ne touchent pas les jours.
    let chart = Stack::with_children(vec![
        container(sparkline::bar_chart(heights))
            .width(Length::Fill)
            .height(176.0)
            .padding(Padding::new(0.0).bottom(space::LG))
            .into(),
        overlay.into(),
    ])
    .width(Length::Fill)
    .height(176.0);

    surface::panel(
        column![
            surface::section_header("Activité récente", Space::with_width(0)),
            surface::divider(),
            container(chart).padding([space::LG, 0.0]),
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
}

/// Panneau « Candidatures récentes » : les cinq plus récentes mises à jour,
/// avec le statut et la date de chaque candidature.
fn recent_panel(app: &App) -> Container<'_, Message> {
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

/// Abréviation d'un jour de la semaine en une lettre (L, M, M, J, V, S, D).
const fn day_letter(index: u32) -> &'static str {
    match index {
        0 => "L",
        1 => "M",
        2 => "M",
        3 => "J",
        4 => "V",
        5 => "S",
        _ => "D",
    }
}

/// Dates des sept derniers jours avec le compte de candidatures envoyées
/// chaque jour, depuis le jour le plus ancien jusqu'à aujourd'hui.
///
/// La comparaison se fait sur le préfixe `AAAA-MM-JJ` : les lignes reprises
/// de l'ancienne base portent un horodatage complet qui commence par la date.
fn last_seven_days(candidates: &[Candidature]) -> Vec<(chrono::NaiveDate, usize)> {
    let today = chrono::Local::now().date_naive();
    let mut days = Vec::with_capacity(7);
    let mut cursor = today.checked_sub_days(chrono::Days::new(6));
    while let Some(date) = cursor {
        let prefix = date.format("%Y-%m-%d").to_string();
        let count = candidates
            .iter()
            .filter(|item| item.date_envoi.starts_with(&prefix))
            .count();
        days.push((date, count));
        cursor = date.succ_opt().filter(|next| next <= &today);
    }
    days
}

#[cfg(test)]
mod tests {
    use super::last_seven_days;
    use crate::modules::candidatures::model::{Candidature, StatutCandidature, TypeContrat};
    use uuid::Uuid;

    fn candidature(date: &str) -> Candidature {
        Candidature {
            id: Uuid::new_v4(),
            poste: "Développeur".into(),
            entreprise_id: Uuid::new_v4(),
            entreprise_nom: Some("Agrial".into()),
            contact_id: None,
            type_contrat: TypeContrat::Cdi,
            statut: StatutCandidature::EnAttente,
            date_envoi: date.into(),
            lien_offre: None,
            notes: None,
            created_at: date.into(),
            updated_at: date.into(),
        }
    }

    #[test]
    fn les_sept_jours_sont_consecutifs_et_finissent_aujourd_hui() {
        let days = last_seven_days(&[]);
        assert_eq!(days.len(), 7);
        for pair in days.windows(2) {
            assert_eq!(
                pair[0].0.succ_opt(),
                Some(pair[1].0),
                "jours non consécutifs"
            );
        }
        assert_eq!(days.last().unwrap().0, chrono::Local::now().date_naive());
    }

    #[test]
    fn les_comptes_ne_portent_que_les_candidatures_de_la_fenetre() {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let candidates = vec![
            candidature(&today),
            candidature(&today),
            candidature("2020-01-01"),
        ];
        let days = last_seven_days(&candidates);
        assert_eq!(days.last().unwrap().1, 2, "les envois du jour");
        assert_eq!(days.first().unwrap().1, 0, "hors fenêtre");
        assert_eq!(
            days.iter().map(|(_, count)| count).sum::<usize>(),
            2,
            "aucune candidature hors fenêtre ne compte"
        );
    }
}
