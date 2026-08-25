//! Panneaux d'activité et d'événements du tableau de bord.
//!
//! Gabarit de la maquette « refonte-design » : jour en pastille, activité par
//! barres verticales, pipeline en quatre étapes et candidatures récentes avec
//! avatar.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::candidatures::components::{contract_short, status_badge};
use crate::modules::metriques::components::PipelineCounts;
use crate::navigation::Route;
use crate::ui::components::avatar;
use crate::ui::components::button as controls;
use crate::ui::components::icon::Icon;
use crate::ui::components::table::{self, Column};
use crate::ui::components::{badge, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::{radius, space};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use chrono::Datelike;
use iced::widget::{column, container, row, Container, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

const RECENT_COLUMNS: [Column; 4] = [
    Column::text("POSTE", 4),
    Column::text("ENTREPRISE", 3).secondary(),
    Column::centered("STATUT", 126.0).secondary(),
    Column::trailing("MISE À JOUR", 104.0).secondary(),
];

/// Panneau « Prochains événements » : entretiens à venir puis relances
/// échues, chacun portant un bloc de jour, un titre, une sous-ligne et un jeton.
pub(super) fn upcoming_panel<'a>(
    app: &'a App,
    today: &str,
    upcoming: usize,
) -> Container<'a, Message> {
    let mut rows = column![].spacing(space::SM);
    let mut count = 0_usize;

    let mut interviews: Vec<_> = app
        .data
        .entretiens
        .iter()
        .filter(|item| item.date_entretien.as_str() >= today)
        .collect();
    interviews.sort_by(|left, right| left.date_entretien.cmp(&right.date_entretien));

    for interview in interviews.into_iter().take(3) {
        count += 1;
        let (poste, company) = app
            .data
            .candidatures
            .iter()
            .find(|item| item.id == interview.candidature_id)
            .map(|item| {
                (
                    item.poste.clone(),
                    item.entreprise_nom.clone().unwrap_or_default(),
                )
            })
            .unwrap_or_else(|| ("Candidature".to_owned(), String::new()));
        let mut subtitle = poste;
        if let Some(lieu) = interview.lieu.as_deref() {
            subtitle = format!("{subtitle} · {lieu}");
        }
        rows = rows.push(agenda_row(
            &interview.date_entretien,
            "Entretien",
            Tone::Success,
            format!("Entretien — {company}"),
            subtitle,
        ));
    }

    let mut reminders: Vec<_> = app
        .data
        .relances
        .iter()
        .filter(|item| item.date_relance.as_str() <= today)
        .collect();
    reminders.sort_by(|left, right| left.date_relance.cmp(&right.date_relance));

    for reminder in reminders.into_iter().take(3) {
        count += 1;
        let (poste, company) = app
            .data
            .candidatures
            .iter()
            .find(|item| item.id == reminder.candidature_id)
            .map(|item| {
                (
                    item.poste.clone(),
                    item.entreprise_nom.clone().unwrap_or_default(),
                )
            })
            .unwrap_or_else(|| ("Candidature".to_owned(), String::new()));
        rows = rows.push(agenda_row(
            &reminder.date_relance,
            "Relance",
            Tone::Warning,
            format!("Relance — {company}"),
            format!("{poste} · sans réponse"),
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
/// barres verticales, avec la valeur au-dessus et le jour sous chaque barre.
pub(super) fn activity_panel(app: &App) -> Container<'_, Message> {
    let days = last_seven_days(&app.data.candidature_stats.activity_by_day);
    let maximum = days
        .iter()
        .map(|(_, count)| *count)
        .max()
        .unwrap_or(1)
        .max(1);
    let mut chart = row![]
        .spacing(space::MD)
        .align_y(Alignment::End)
        .width(Length::Fill);
    for (date, count) in &days {
        let pct = (*count as f32 / maximum as f32) * 100.0;
        let bar_height = 4.0 + pct * 0.92;
        let day_label = format::weekday_abbrev(date.weekday().num_days_from_monday());
        let is_zero = *count == 0;
        let value_style = move |theme: &Theme| {
            if is_zero {
                styles::muted_text(theme)
            } else {
                styles::toned_text(Tone::Accent)(theme)
            }
        };
        let bar_style = move |theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(if is_zero {
                    palette.neutral_tint
                } else {
                    palette.accent
                })),
                border: Border {
                    radius: radius::CONTROL.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }
        };
        chart = chart.push(
            column![
                typo::caption(count.to_string()).style(value_style),
                container(Space::new(Length::Fixed(22.0), Length::Fixed(bar_height)))
                    .width(Length::Fixed(22.0))
                    .height(Length::Fixed(bar_height))
                    .style(bar_style),
                typo::caption(day_label).style(styles::muted_text),
            ]
            .spacing(space::XS)
            .align_x(Alignment::Center)
            .width(Length::Fill),
        );
    }

    surface::panel(
        column![
            surface::section_header(
                "Activité récente",
                badge::badge("7 derniers jours", Tone::Neutral),
            ),
            surface::divider(),
            container(chart)
                .padding([space::MD, space::XS])
                .width(Length::Fill),
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
}

/// Panneau « Pipeline » : les quatre étapes du suivi avec leur compte.
pub(super) fn pipeline_panel(counts: &PipelineCounts) -> Container<'static, Message> {
    let conversion = counts
        .interviews
        .checked_mul(100)
        .and_then(|value| value.checked_div(counts.total))
        .unwrap_or(0);
    let stages = [
        ("En attente", counts.pending, Tone::Neutral),
        ("Relancées", counts.followed_up, Tone::Warning),
        ("Entretiens", counts.interviews, Tone::Success),
        ("Refusées", counts.rejected, Tone::Danger),
    ];
    let mut list = column![].spacing(space::SM);
    for (label, value, tone) in stages {
        let dot_style = move |theme: &Theme| container::Style {
            background: Some(Background::Color(tone.color(&tokens(theme)))),
            border: Border {
                radius: radius::PILL.into(),
                ..Border::default()
            },
            ..container::Style::default()
        };
        list = list.push(
            container(
                row![
                    container(Space::new(Length::Fixed(8.0), Length::Fixed(8.0))).style(dot_style),
                    typo::body(label),
                    iced::widget::Space::with_width(Length::Fill),
                    typo::text_mono(value.to_string(), font::BODY, font::MONO_SEMIBOLD),
                ]
                .spacing(space::SM)
                .align_y(Alignment::Center),
            )
            .padding([space::SM, space::MD])
            .style(styles::glass_card),
        );
    }

    surface::panel(
        column![
            surface::section_header(
                "Pipeline",
                badge::badge(format!("Conversion {conversion} %"), Tone::Accent),
            ),
            surface::divider(),
            container(list)
                .padding([space::MD, 0.0])
                .width(Length::Fill),
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
        let company = format::or_else(candidate.entreprise_nom.as_deref(), "Entreprise inconnue");
        let initials = avatar::initials_of(&company);
        let cells = vec![
            table::cell(
                RECENT_COLUMNS[0],
                row![
                    avatar::avatar(initials, 28.0, Tone::Accent),
                    column![
                        typo::item(format::truncate(&candidate.poste, 40)),
                        typo::caption(contract_short(candidate.type_contrat)),
                    ]
                    .spacing(0),
                ]
                .spacing(space::SM)
                .align_y(Alignment::Center),
            ),
            table::cell(
                RECENT_COLUMNS[1],
                typo::meta(format::truncate(&company, 42)),
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

/// Ligne d'agenda au gabarit de la maquette : bloc de jour, titre, sous-ligne
/// et jeton de nature (Entretien / Relance).
fn agenda_row<'a, Message: 'a>(
    date: &str,
    kind: &'static str,
    tone: Tone,
    title: String,
    subtitle: String,
) -> Element<'a, Message> {
    let (day, month) = date_day_month(date);
    row![
        day_block(day, month, tone),
        column![
            typo::item(title),
            typo::caption(subtitle).style(styles::muted_text),
        ]
        .spacing(space::XXS)
        .width(Length::Fill),
        badge::badge(kind, tone),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center)
    .into()
}

/// Extrait le numéro de jour et le mois (en toutes lettres, minuscule) d'une
/// date ISO `AAAA-MM-JJ`.
fn date_day_month(date: &str) -> (String, String) {
    let prefix = &date[..date.len().min(10)];
    match chrono::NaiveDate::parse_from_str(prefix, "%Y-%m-%d") {
        Ok(parsed) => (
            parsed.format("%d").to_string(),
            format::month_name(parsed.month()).to_lowercase(),
        ),
        Err(_) => ("–".to_owned(), String::new()),
    }
}

/// Bloc de jour 44 × 46 : numéro en graisse mono et mois, sur fond teinté.
fn day_block<'a, Message: 'a>(day: String, month: String, tone: Tone) -> Element<'a, Message> {
    let month_label: Element<'a, Message> = if month.is_empty() {
        iced::widget::Space::with_height(0).into()
    } else {
        typo::caption(month).style(styles::muted_text).into()
    };
    container(
        column![
            typo::text_mono(day, 16.0, font::MONO_SEMIBOLD).style(styles::toned_text(tone)),
            month_label,
        ]
        .spacing(0)
        .align_x(Alignment::Center),
    )
    .width(44.0)
    .height(46.0)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
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
