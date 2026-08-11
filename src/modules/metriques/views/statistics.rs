//! Écran Statistiques : onglet Candidatures (barres, donut, entonnoir,
//! relances) et onglet Performance CV (scores ATS et appels IA paginés).

use crate::app::state::StatisticsTab;
use crate::app::{App, Message};
use crate::modules::candidatures::components::{column_label, status_tone, PIPELINE};
use crate::modules::candidatures::model::Candidature;
use crate::modules::metriques::components::PipelineCounts;
use crate::modules::metriques::model::{OrigineScore, ResumeScoresAts};
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::table::Column as TableColumn;
use crate::ui::components::tabs::Tab;
use crate::ui::components::{
    badge, bar, donut, header, layout, list, pagination, sparkline, stat_card, state, surface,
    table, tabs, typo,
};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use chrono::{Days, NaiveDate};
use iced::widget::{column, container, row, Container, Space, Stack};
use iced::{Alignment, Element, Length, Padding};

/// Nombre de lignes par page des historiques ATS et IA.
pub const PAGE_SIZE: u64 = 10;

/// Colonnes de l'historique des scores ATS.
const ATS_COLUMNS: [TableColumn; 3] = [
    TableColumn::text("DATE", 3),
    TableColumn::text("ORIGINE", 3),
    TableColumn::trailing("SCORE", 84.0),
];

/// Colonnes de l'historique des appels IA.
const LLM_COLUMNS: [TableColumn; 5] = [
    TableColumn::text("OPÉRATION", 3),
    TableColumn::text("FOURNISSEUR", 2),
    TableColumn::text("MODÈLE", 2),
    TableColumn::trailing("LATENCE", 88.0),
    TableColumn::centered("SUCCÈS", 76.0),
];

/// Rend l'écran des statistiques.
pub fn view(app: &App) -> Element<'_, Message> {
    let counts = PipelineCounts::from_candidates(&app.data.candidatures);
    layout::screen(
        header::page_header(
            Icon::Chart,
            "Statistiques",
            format::plural(counts.total, "candidature suivie", "candidatures suivies"),
            Space::with_width(0).into(),
        ),
        layout::workspace(
            column![
                tabs::tabs(
                    [
                        Tab::new(
                            "Candidatures",
                            app.statistics_tab == StatisticsTab::Candidatures,
                        ),
                        Tab::new(
                            "Performance CV",
                            app.statistics_tab == StatisticsTab::PerformanceCv,
                        ),
                    ],
                    |index| Message::StatisticsTabChanged(if index == 0 {
                        StatisticsTab::Candidatures
                    } else {
                        StatisticsTab::PerformanceCv
                    }),
                ),
                match app.statistics_tab {
                    StatisticsTab::Candidatures => candidatures_tab(app, &counts),
                    StatisticsTab::PerformanceCv => performance_tab(app),
                },
            ]
            .spacing(space::LG)
            .height(Length::Fill),
        ),
    )
}

// --------------------------------------------------------------------------
// Onglet Candidatures
// --------------------------------------------------------------------------

/// Hauteur réservée à la rangée de panneaux graphiques.
///
/// Couvre l'en-tête de panneau, son filet, ses marges et le tracé lui-même (`.height(176.0)`),
/// de sorte que le graphique ne soit jamais rogné quelle que soit la hauteur de la fenêtre.
const HAUTEUR_PANNEAUX: f32 = 300.0;

fn candidatures_tab<'a>(app: &'a App, counts: &PipelineCounts) -> Element<'a, Message> {
    let today = chrono::Local::now().date_naive();
    let due = app.due_reminders(&today.format("%Y-%m-%d").to_string());
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
                counts.interviews.to_string(),
                Tone::Violet,
                Icon::Calendar,
            ),
            stat_card::metric_icon_tinted(
                "Taux d'entretien",
                format!("{} %", interview_rate(counts)),
                Tone::Success,
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
        layout::columns_of_height(
            HAUTEUR_PANNEAUX,
            [
                activity_panel(app, today)
                    .width(Length::FillPortion(3))
                    .into(),
                funnel_panel(counts).width(Length::FillPortion(2)).into(),
            ],
        ),
        reminders_band(app, today, due),
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
    let counts = weekly_counts(&app.data.candidatures, today);
    let heights = sparkline::bar_heights(counts);

    let mut overlay = row![].width(Length::Fill).height(Length::Fill);
    for (index, count) in counts.iter().enumerate() {
        overlay = overlay.push(
            column![
                typo::text_mono(count.to_string(), 11.0, font::MONO_REGULAR),
                Space::with_height(Length::Fill),
                typo::caption(week_label(today, index)),
            ]
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .align_x(Alignment::Center),
        );
    }

    // Un seul canvas dessine les barres ; une grille de textes par-dessus lui
    // ajoute la valeur (en haut) et le début de semaine (en bas).
    let chart = Stack::with_children(vec![
        container(sparkline::bar_chart_n(&heights))
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
            surface::section_header(
                "Candidatures envoyées",
                badge::badge(
                    format!(
                        "{} actions · 30 j",
                        recent_actions(&app.data.candidatures, today)
                    ),
                    Tone::Neutral,
                ),
            ),
            surface::divider(),
            container(chart).padding([space::LG, 0.0]),
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
}

/// Panneau donut + entonnoir : le taux d'entretien au centre, la répartition
/// par statut en barres.
fn funnel_panel<'a>(counts: &PipelineCounts) -> Container<'a, Message> {
    let body: Element<'a, Message> = if counts.total == 0 {
        state::empty(
            "Pas encore de données",
            "L'entonnoir apparaîtra dès la première candidature enregistrée.",
        )
    } else {
        let rate = interview_rate(counts);
        let ratio = counts.interviews as f32 / counts.total as f32;
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
                container(donut::donut(ratio, 112.0, Tone::Success)).into(),
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
fn reminders_band<'a>(app: &'a App, today: NaiveDate, due: usize) -> Element<'a, Message> {
    let today_str = today.format("%Y-%m-%d").to_string();
    let mut reminders: Vec<_> = app
        .data
        .relances
        .iter()
        .filter(|item| item.date_relance.as_str() <= today_str.as_str())
        .collect();
    reminders.sort_by(|left, right| left.date_relance.cmp(&right.date_relance));

    let mut rows = column![].spacing(0);
    for reminder in reminders.iter().take(5) {
        let poste = app
            .data
            .candidatures
            .iter()
            .find(|item| item.id == reminder.candidature_id)
            .map_or_else(|| "Candidature".to_owned(), |item| item.poste.clone());
        rows = rows.push(list::row_static(
            alert_icon(),
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

    let list: Element<'a, Message> = if reminders.is_empty() {
        typo::caption("Aucune relance arrivée à échéance.").into()
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
// --------------------------------------------------------------------------

fn performance_tab<'a>(app: &'a App) -> Element<'a, Message> {
    let summary = app.data.ats_summary.as_ref();
    let corps = column![
        row![
            stat_card::metric_icon_tinted(
                "Score moyen",
                format!("{} / 100", summary.map_or(0, |item| item.moyenne)),
                Tone::Accent,
                Icon::Chart,
            ),
            stat_card::metric_icon_tinted(
                "Scores évalués",
                summary.map_or(0, |item| item.nombre).to_string(),
                Tone::Violet,
                Icon::Document,
            ),
            stat_card::metric_icon_tinted(
                "Générés",
                summary.map_or(0, |item| item.generes_nombre).to_string(),
                Tone::Info,
                Icon::Sparkles,
            ),
            stat_card::metric_icon_tinted(
                "Importés",
                summary.map_or(0, |item| item.importes_nombre).to_string(),
                Tone::Success,
                Icon::Import,
            ),
        ]
        .spacing(space::MD),
        layout::columns_of_height(
            HAUTEUR_PANNEAUX,
            [
                distribution_panel(summary)
                    .width(Length::FillPortion(1))
                    .into(),
                history_panel(app).width(Length::FillPortion(2)).into(),
            ],
        ),
        calls_panel(app),
    ]
    .spacing(space::LG);
    // Même structure à trois blocs que `candidatures_tab`, donc même correctif.
    surface::scroll(corps).height(Length::Fill).into()
}

/// Répartition des scores ATS en quatre tranches colorées.
fn distribution_panel<'a>(summary: Option<&ResumeScoresAts>) -> Container<'a, Message> {
    let nombre = summary.map_or(0, |item| item.nombre);
    let ratio = |part: u64| {
        if nombre == 0 {
            0.0
        } else {
            part as f32 / nombre as f32 * 100.0
        }
    };
    let mut bars = column![].spacing(space::LG);
    for (label, part, tone) in [
        (
            "Faibles · 0–49",
            summary.map_or(0, |item| item.faibles),
            Tone::Danger,
        ),
        (
            "Partiels · 50–69",
            summary.map_or(0, |item| item.partiels),
            Tone::Warning,
        ),
        (
            "Bons · 70–84",
            summary.map_or(0, |item| item.bons),
            Tone::Success,
        ),
        (
            "Excellents · 85–100",
            summary.map_or(0, |item| item.excellents),
            Tone::Accent,
        ),
    ] {
        bars = bars.push(bar::barre(label, part.to_string(), ratio(part), tone));
    }

    surface::panel(
        column![
            surface::section_header("Répartition des scores", badge::count(nombre as usize)),
            surface::divider(),
            container(bars).padding([space::LG, 0.0]),
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
}

/// Historique des scores ATS : table paginée, date, origine et score.
fn history_panel<'a>(app: &'a App) -> Container<'a, Message> {
    // La page vient de la base (`LIMIT`/`OFFSET`) : plus de découpage en mémoire d'un
    // historique intégralement chargé.
    let all = &app.data.ats_scores;
    let body: Element<'a, Message> = if all.items.is_empty() {
        state::empty_slot("Aucun score ATS enregistré.")
    } else {
        let mut rows = column![];
        for score in &all.items {
            rows = rows.push(table::row_static(
                app.layout(),
                [
                    table::cell(
                        ATS_COLUMNS[0],
                        typo::text_mono(
                            format::compact_date(&score.cree_le),
                            12.0,
                            font::MONO_REGULAR,
                        ),
                    ),
                    table::cell(
                        ATS_COLUMNS[1],
                        badge::badge(origine_label(score.origine), Tone::Neutral),
                    ),
                    table::cell(
                        ATS_COLUMNS[2],
                        typo::text_mono(format!("{}/100", score.score), 13.0, font::MONO_SEMIBOLD),
                    ),
                ],
            ));
        }
        let total = all.total;
        let footer: Element<'a, Message> = if all.total_pages > 1 {
            let (first, last) = pagination::window(app.ats_page, PAGE_SIZE, total);
            pagination::pagination(
                app.ats_page,
                all.total_pages,
                Message::AtsPagePrev,
                Message::AtsPageNext,
                first,
                last,
                total,
            )
        } else {
            Space::with_height(0).into()
        };
        column![
            table::header(app.layout(), &ATS_COLUMNS),
            surface::scroll(rows).height(Length::Fill),
            footer,
        ]
        .height(Length::Fill)
        .into()
    };

    surface::panel(
        column![
            surface::section_header(
                "Historique des scores",
                badge::count(usize::try_from(all.total).unwrap_or(usize::MAX))
            ),
            surface::divider(),
            body,
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
}

/// Historique des appels IA : table paginée, opération, fournisseur,
/// modèle, latence et succès.
fn calls_panel<'a>(app: &'a App) -> Container<'a, Message> {
    let all = &app.data.llm_calls;
    let body: Element<'a, Message> = if all.items.is_empty() {
        state::empty_slot("Aucun appel IA enregistré.")
    } else {
        let mut rows = column![];
        for call in &all.items {
            let status = if call.succes {
                icon::icon(Icon::Check, icon::SM, Ink::Toned(Tone::Success))
            } else {
                icon::icon(Icon::Alert, icon::SM, Ink::Toned(Tone::Danger))
            };
            rows = rows.push(table::row_static(
                app.layout(),
                [
                    table::cell(LLM_COLUMNS[0], typo::body(operation_label(call.operation))),
                    table::cell(LLM_COLUMNS[1], typo::body(call.provider.clone())),
                    table::cell(LLM_COLUMNS[2], typo::caption(call.modele.clone())),
                    table::cell(
                        LLM_COLUMNS[3],
                        typo::text_mono(
                            format!("{} ms", call.latence_ms),
                            12.0,
                            font::MONO_REGULAR,
                        ),
                    ),
                    table::cell(LLM_COLUMNS[4], status),
                ],
            ));
        }
        let total = all.total;
        let footer: Element<'a, Message> = if all.total_pages > 1 {
            let (first, last) = pagination::window(app.llm_page, PAGE_SIZE, total);
            pagination::pagination(
                app.llm_page,
                all.total_pages,
                Message::LlmPagePrev,
                Message::LlmPageNext,
                first,
                last,
                total,
            )
        } else {
            Space::with_height(0).into()
        };
        column![
            table::header(app.layout(), &LLM_COLUMNS),
            surface::scroll(rows).height(Length::Fill),
            footer,
        ]
        .height(Length::Fill)
        .into()
    };

    surface::panel(
        column![
            surface::section_header(
                "Appels IA",
                badge::count(usize::try_from(all.total).unwrap_or(usize::MAX))
            ),
            surface::divider(),
            body,
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
}

/// Libellé français d'une opération IA, pour l'historique.
fn operation_label(operation: crate::modules::metriques::model::OperationLlm) -> &'static str {
    match operation {
        crate::modules::metriques::model::OperationLlm::ParseOffer => "Analyse d'offre",
        crate::modules::metriques::model::OperationLlm::GenerateCv => "Génération CV",
        crate::modules::metriques::model::OperationLlm::AnalyzeAts => "Analyse ATS",
        crate::modules::metriques::model::OperationLlm::ParseCv => "Import CV",
        crate::modules::metriques::model::OperationLlm::AnalyserEntretien => "Compte rendu",
        crate::modules::metriques::model::OperationLlm::CoverLetter => "Lettre de motivation",
    }
}

/// Libellé français de l'origine d'un score ATS.
const fn origine_label(origine: OrigineScore) -> &'static str {
    match origine {
        OrigineScore::Genere => "Généré",
        OrigineScore::Importe => "Importé",
    }
}

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

/// Taux d'entretien (0-100), arrondi.
fn interview_rate(counts: &PipelineCounts) -> u8 {
    if counts.total == 0 {
        return 0;
    }
    ((counts.interviews as f64 / counts.total as f64) * 100.0).round() as u8
}

/// Candidatures envoyées au cours des 30 derniers jours.
fn recent_actions(candidates: &[Candidature], today: NaiveDate) -> usize {
    let threshold = today.checked_sub_days(Days::new(30)).unwrap_or(today);
    candidates
        .iter()
        .filter(|candidate| {
            candidate_date(&candidate.date_envoi)
                .is_some_and(|date| date >= threshold && date <= today)
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::{interview_rate, weekly_counts};
    use crate::modules::candidatures::model::{Candidature, StatutCandidature, TypeContrat};
    use crate::modules::metriques::components::PipelineCounts;
    use chrono::NaiveDate;

    fn candidature(date: &str) -> Candidature {
        Candidature {
            id: uuid::Uuid::new_v4(),
            poste: "Développeur".into(),
            entreprise_id: uuid::Uuid::new_v4(),
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
    fn les_huit_semaines_s_ordonnent_de_la_plus_ancienne_a_la_courante() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
        let candidates = vec![
            candidature("2026-08-10"),
            candidature("2026-08-03"),
            candidature("2026-07-20"),
        ];
        let counts = weekly_counts(&candidates, today);
        assert_eq!(counts[7], 1, "la semaine courante");
        assert_eq!(counts[6], 1, "la semaine précédente");
        assert_eq!(counts[4], 1, "il y a trois semaines");
        assert_eq!(counts.iter().sum::<usize>(), 3);
    }

    #[test]
    fn les_bornes_de_la_fenetre_des_huit_semaines_sont_respectees() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
        let candidates = vec![
            candidature("2026-08-09"),
            candidature("2026-06-16"), // 55 jours : dernière semaine incluse
            candidature("2026-06-15"), // 56 jours : hors fenêtre
            candidature("2026-08-11"), // future : ignorée
            candidature("pas-une-date"),
        ];
        let counts = weekly_counts(&candidates, today);
        assert_eq!(counts[7], 1);
        assert_eq!(counts[0], 1);
        assert_eq!(counts.iter().sum::<usize>(), 2);
    }

    #[test]
    fn un_horodatage_complet_est_compte_par_son_prefixe_de_date() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
        let candidates = vec![candidature("2026-08-10T14:30:00Z")];
        let counts = weekly_counts(&candidates, today);
        assert_eq!(counts[7], 1);
    }

    #[test]
    fn le_taux_d_entretien_est_un_pourcentage_arrondi_sans_division_par_zero() {
        let counts = PipelineCounts {
            total: 10,
            interviews: 3,
            ..PipelineCounts::default()
        };
        assert_eq!(interview_rate(&counts), 30);
        assert_eq!(interview_rate(&PipelineCounts::default()), 0);
        let complet = PipelineCounts {
            total: 4,
            interviews: 4,
            ..PipelineCounts::default()
        };
        assert_eq!(interview_rate(&complet), 100);
    }
}
