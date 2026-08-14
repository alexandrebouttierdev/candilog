//! Statistiques des scores ATS et de l'utilisation IA.

use crate::app::{App, Message};
use crate::modules::metriques::model::{OrigineScore, ResumeScoresAts};
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::table::Column as TableColumn;
use crate::ui::components::{
    badge, bar, layout, pagination, stat_card, state, surface, table, typo,
};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, Container, Space};
use iced::{Element, Length};

use super::PAGE_SIZE;

const HAUTEUR_PANNEAUX: f32 = 300.0;

const ATS_COLUMNS: [TableColumn; 3] = [
    TableColumn::text("DATE", 3),
    TableColumn::text("ORIGINE", 3),
    TableColumn::trailing("SCORE", 84.0),
];

const LLM_COLUMNS: [TableColumn; 5] = [
    TableColumn::text("OPÉRATION", 3),
    TableColumn::text("FOURNISSEUR", 2),
    TableColumn::text("MODÈLE", 2),
    TableColumn::trailing("LATENCE", 88.0),
    TableColumn::centered("SUCCÈS", 76.0),
];

// --------------------------------------------------------------------------

pub(super) fn performance_tab<'a>(app: &'a App) -> Element<'a, Message> {
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
                Tone::Info,
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
        calls_panel(app).height(Length::Fixed(HAUTEUR_PANNEAUX)),
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
