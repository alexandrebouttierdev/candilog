//! Statistiques du pipeline de candidatures et des relances.

use crate::app::{App, Message};
use crate::modules::candidatures::components::{column_label, status_tone, PIPELINE};
#[cfg(test)]
use crate::modules::candidatures::model::Candidature;
use crate::modules::metriques::components::PipelineCounts;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::{badge, bar, donut, layout, list, stat_card, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use chrono::{Days, NaiveDate};
use iced::widget::{column, container, row, Container, Stack};
use iced::{Alignment, Element, Length};

// --------------------------------------------------------------------------
// Onglet Candidatures
// --------------------------------------------------------------------------

/// Hauteur réservée à la rangée de panneaux graphiques.
///
/// Couvre l'en-tête de panneau, son filet, ses marges et le tracé lui-même (`.height(176.0)`),
/// de sorte que le graphique ne soit jamais rogné quelle que soit la hauteur de la fenêtre.
const HAUTEUR_PANNEAUX: f32 = 300.0;

pub(super) fn candidatures_tab<'a>(app: &'a App, counts: &PipelineCounts) -> Element<'a, Message> {
    let today = chrono::Local::now().date_naive();
    let due = usize::try_from(app.data.candidature_stats.to_follow_up).unwrap_or(usize::MAX);
    let conversions =
        usize::try_from(app.data.candidature_stats.converted_candidates).unwrap_or(usize::MAX);
    let corps = column![
        row![
            stat_card::metric_icon_tinted(
                "Candidatures",
                counts.total.to_string(),
                Tone::Accent,
                Icon::Applications,
            ),
            stat_card::metric_icon_tinted(
                "Entretiens",
                conversions.to_string(),
                Tone::Info,
                Icon::Calendar,
            ),
            stat_card::metric_icon_tinted(
                "Taux de conversion",
                format!("{} %", interview_rate(counts.total, conversions)),
                Tone::Violet,
                Icon::Chart,
            ),
            stat_card::metric_icon_tinted(
                "Relances à faire",
                due.to_string(),
                Tone::Warning,
                Icon::Alert,
            ),
        ]
        .spacing(space::MD),
        row![
            stat_card::metric_icon_tinted(
                "Candidatures actives",
                counts.active().to_string(),
                Tone::Accent,
                Icon::Target,
            ),
            stat_card::metric_icon_tinted(
                "Taux de réponse",
                format!("{} %", counts.response_rate()),
                Tone::Info,
                Icon::Chart,
            ),
            stat_card::metric_icon_tinted(
                "Refus reçus",
                counts.rejected.to_string(),
                Tone::Danger,
                Icon::Close,
            ),
        ]
        .spacing(space::MD),
        layout::columns_of_height(
            HAUTEUR_PANNEAUX,
            [
                activity_panel(app, today)
                    .width(Length::FillPortion(3))
                    .into(),
                funnel_panel(app, counts)
                    .width(Length::FillPortion(2))
                    .into(),
            ],
        ),
        reminders_band(app, due),
    ]
    .spacing(space::LG);
    // L'onglet défile au lieu de comprimer. Les trois blocs étaient empilés dans une colonne
    // de hauteur `Fill` : la rangée de graphiques, seul enfant élastique, ne recevait que
    // l'espace laissé par le bandeau des relances, dont la hauteur croît avec le nombre de
    // relances dues. Passé cinq relances, elle devenait plus courte que la hauteur fixe du
    // graphique et le contenu était **rogné sans le moindre indice** : ni ellipse, ni barre de
    // défilement, ni message — les deux panneaux finissaient vides, réduits à leur titre.
    surface::scroll(corps).height(Length::Fill).into()
}

/// Panneau des barres : candidatures envoyées sur les 8 dernières semaines.
fn activity_panel<'a>(app: &'a App, today: NaiveDate) -> Container<'a, Message> {
    let counts = weekly_counts_from_activity(&app.data.candidature_stats.activity_by_day, today);
    let maximum = counts.iter().copied().max().unwrap_or(1).max(1);
    let mut chart = column![].spacing(space::SM).width(Length::Fill);
    for (index, count) in counts.iter().enumerate() {
        chart = chart.push(bar::barre(
            week_label(today, index),
            format::plural(*count, "candidature", "candidatures"),
            (*count as f32 / maximum as f32) * 100.0,
            Tone::Accent,
        ));
    }

    surface::panel(
        column![
            surface::section_header(
                "Candidatures envoyées",
                badge::badge(
                    format!(
                        "{} actions · 30 j",
                        recent_actions_from_activity(
                            &app.data.candidature_stats.activity_by_day,
                            today,
                        )
                    ),
                    Tone::Neutral,
                ),
            ),
            surface::divider(),
            container(chart).padding([space::MD, 0.0]),
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
}

/// Panneau donut + entonnoir : le taux de conversion au centre, la répartition
/// par statut en barres.
fn funnel_panel<'a>(app: &App, counts: &PipelineCounts) -> Container<'a, Message> {
    let body: Element<'a, Message> = if counts.total == 0 {
        state::empty(
            "Pas encore de données",
            "L'entonnoir apparaîtra dès la première candidature enregistrée.",
        )
    } else {
        let conversions =
            usize::try_from(app.data.candidature_stats.converted_candidates).unwrap_or(usize::MAX);
        let rate = interview_rate(counts.total, conversions);
        let ratio = conversions.min(counts.total) as f32 / counts.total as f32;
        let mut bars = column![].spacing(space::LG);
        for status in PIPELINE {
            let value = match status {
                crate::modules::candidatures::model::StatutCandidature::EnAttente => counts.pending,
                crate::modules::candidatures::model::StatutCandidature::Relancee => {
                    counts.followed_up
                }
                crate::modules::candidatures::model::StatutCandidature::Entretien => {
                    counts.interviews
                }
                crate::modules::candidatures::model::StatutCandidature::Refus => counts.rejected,
            };
            bars = bars.push(bar::barre(
                column_label(status),
                value.to_string(),
                value as f32 / counts.total as f32 * 100.0,
                status_tone(status),
            ));
        }
        row![
            Stack::with_children(vec![
                container(donut::donut(ratio, 112.0, Tone::Violet)).into(),
                container(donut::center(format!("{rate} %")))
                    .width(112.0)
                    .height(112.0)
                    .center_x(112.0)
                    .center_y(112.0)
                    .into(),
            ])
            .width(112.0)
            .height(112.0),
            container(bars).width(Length::Fill),
        ]
        .spacing(space::XL)
        .align_y(Alignment::Center)
        .into()
    };

    surface::panel(
        column![
            surface::section_header("Entonnoir", badge::count(counts.total)),
            surface::divider(),
            container(body).padding([space::LG, 0.0]),
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
}

/// Bandeau ambre : les relances arrivées à échéance, avec leur compteur.
fn reminders_band<'a>(app: &'a App, due: usize) -> Element<'a, Message> {
    let mut rows = column![].spacing(0);
    for candidate in app.data.follow_up_candidates.iter().take(5) {
        rows = rows.push(list::row_item(
            alert_icon(),
            format::truncate(&candidate.poste, 30),
            format::compact_date(&candidate.date_envoi),
            typo::caption(
                candidate
                    .entreprise_nom
                    .clone()
                    .unwrap_or_else(|| "Entreprise non renseignée".to_owned()),
            ),
            false,
            Message::OpenCandidateFromStats(candidate.id),
        ));
    }

    let list: Element<'a, Message> = if app.data.follow_up_candidates.is_empty() {
        typo::caption("Aucune candidature sans réponse depuis plus de 7 jours.").into()
    } else {
        rows.into()
    };

    container(
        column![
            surface::section_header(
                "Candidatures à relancer",
                badge::count_toned(due, Tone::Warning),
            ),
            surface::divider(),
            container(list).padding([space::LG, 0.0]),
        ]
        .width(Length::Fill),
    )
    .padding(space::XL)
    .width(Length::Fill)
    .style(styles::amber_band)
    .into()
}

/// Pastille d'alerte 36 px : icône ambre sur fond teinté du même ton.
fn alert_icon<'a, Message: 'a>() -> Element<'a, Message> {
    container(icon::icon(Icon::Alert, icon::MD, Ink::Toned(Tone::Warning)))
        .width(36.0)
        .height(36.0)
        .center(Length::Fixed(36.0))
        .style(styles::toned(Tone::Warning))
        .into()
}

// --------------------------------------------------------------------------
// Onglet Performance CV
/// Date de la candidature, ignorée quand elle n'est pas lisible.
fn candidate_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value.get(..10).unwrap_or(value), "%Y-%m-%d").ok()
}

/// Début de la semaine d'index donné (0 = la plus ancienne), en `jj/mm`.
fn week_label(today: NaiveDate, index: usize) -> String {
    let days_back = (7 - index) as u64 * 7 + 6;
    let start = today
        .checked_sub_days(Days::new(days_back))
        .unwrap_or(today);
    format::compact_date(&start.format("%Y-%m-%d").to_string())
}

// --------------------------------------------------------------------------
// Helpers purs
// --------------------------------------------------------------------------

/// Comptes hebdomadaires des candidatures envoyées (8 dernières semaines).
///
/// Chaque fenêtre couvre 7 jours glissants : l'index 7 est la semaine
/// courante, l'index 0 la plus ancienne de la fenêtre. La comparaison se
/// fait sur le préfixe `AAAA-MM-JJ` pour les horodatages complets.
#[cfg(test)]
fn weekly_counts(candidates: &[Candidature], today: NaiveDate) -> [usize; 8] {
    let mut counts = [0_usize; 8];
    for candidate in candidates {
        let Some(date) = candidate_date(&candidate.date_envoi) else {
            continue;
        };
        let days = (today - date).num_days();
        if !(0..56).contains(&days) {
            continue;
        }
        counts[7 - days as usize / 7] += 1;
    }
    counts
}

fn weekly_counts_from_activity(activity: &[(String, u64)], today: NaiveDate) -> [usize; 8] {
    let mut counts = [0_usize; 8];
    for (value, count) in activity {
        let Some(date) = candidate_date(value) else {
            continue;
        };
        let days = (today - date).num_days();
        if (0..56).contains(&days) {
            counts[7 - days as usize / 7] = counts[7 - days as usize / 7]
                .saturating_add(usize::try_from(*count).unwrap_or(usize::MAX));
        }
    }
    counts
}

/// Taux d'entretien (0-100), arrondi.
fn interview_rate(total: usize, interviews: usize) -> u8 {
    if total == 0 {
        return 0;
    }
    ((interviews.min(total) as f64 / total as f64) * 100.0).round() as u8
}

/// Candidatures envoyées au cours des 30 derniers jours.
fn recent_actions_from_activity(activity: &[(String, u64)], today: NaiveDate) -> usize {
    let threshold = today.checked_sub_days(Days::new(30)).unwrap_or(today);
    activity
        .iter()
        .filter_map(|(value, count)| candidate_date(value).map(|date| (date, count)))
        .filter(|(date, _)| *date >= threshold && *date <= today)
        .map(|(_, count)| usize::try_from(*count).unwrap_or(usize::MAX))
        .sum()
}

#[cfg(test)]
#[path = "tests/candidatures/mod.rs"]
mod tests;
