//! Écran Statistiques : chaque graphique répond à une question énoncée.

use crate::app::{App, Message};
use crate::modules::candidatures::components::{column_label, status_tone, PIPELINE};
use crate::modules::metriques::components::{LlmSummary, PipelineCounts};
use crate::ui::components::table::Column;
use crate::ui::components::{
    badge, inspector, layout, meter, state, surface, table, toolbar, typo,
};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Colonnes de la table des fournisseurs IA.
const PROVIDER_COLUMNS: [Column; 3] = [
    Column::text("FOURNISSEUR", 3),
    Column::trailing("APPELS", 80.0),
    Column::trailing("PART", 72.0),
];

/// Rend l'écran des statistiques.
pub fn view(app: &App) -> Element<'_, Message> {
    let counts = PipelineCounts::from_candidates(&app.data.candidatures);
    let llm = LlmSummary::from_calls(&app.data.llm_calls);
    let ats_average = app
        .data
        .ats_summary
        .as_ref()
        .map_or(0, |summary| summary.moyenne);

    let body = layout::workspace(
        column![
            headline(counts),
            layout::columns([funnel_panel(counts), quality_panel(app, ats_average, &llm),]),
        ]
        .spacing(space::LG)
        .height(Length::Fill),
    );

    layout::screen(
        toolbar::toolbar(
            "Statistiques",
            typo::meta(format::plural(
                counts.total,
                "candidature suivie",
                "candidatures suivies",
            )),
            iced::widget::Space::with_width(0),
        ),
        body,
    )
}

fn headline<'a>(counts: PipelineCounts) -> Element<'a, Message> {
    surface::panel(
        row![
            meter::metric("Candidatures", counts.total.to_string(), Tone::Accent),
            layout::spacer(),
            meter::metric(
                "Entretiens",
                counts.interviews.to_string(),
                if counts.interviews > 0 {
                    Tone::Success
                } else {
                    Tone::Neutral
                },
            ),
            layout::spacer(),
            meter::metric(
                "Conversion",
                format!("{} %", counts.conversion_rate()),
                Tone::Neutral,
            ),
            layout::spacer(),
            meter::metric(
                "Refus",
                counts.rejected.to_string(),
                if counts.rejected > 0 {
                    Tone::Danger
                } else {
                    Tone::Neutral
                },
            ),
        ]
        .spacing(space::MAX)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .into()
}

fn funnel_panel<'a>(counts: PipelineCounts) -> Element<'a, Message> {
    let body: Element<'a, Message> = if counts.total == 0 {
        state::empty(
            "Pas encore de données",
            "L'entonnoir apparaîtra dès la première candidature enregistrée.",
        )
    } else {
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
            bars = bars.push(meter::bar(
                column_label(status).to_owned(),
                value,
                counts.total,
                status_tone(status),
            ));
        }
        bars = bars
            .push(surface::divider())
            .push(inspector::property_toned(
                "Taux de réponse",
                format!("{} %", counts.response_rate()),
                Tone::Accent,
            ));
        container(bars).padding([space::LG, 0.0]).into()
    };

    surface::panel(
        column![
            surface::section_header("Où en sont mes candidatures ?", badge::count(counts.total),),
            surface::divider(),
            body,
        ]
        .height(Length::Fill),
    )
    .width(Length::FillPortion(3))
    .height(Length::Fill)
    .into()
}

fn quality_panel<'a>(app: &'a App, ats_average: u8, llm: &LlmSummary) -> Element<'a, Message> {
    let providers: Element<'a, Message> = if llm.by_provider.is_empty() {
        state::empty_slot("Aucun appel IA enregistré.")
    } else {
        let mut rows = column![];
        for (provider, calls) in &llm.by_provider {
            rows = rows.push(table::row_static(
                app.layout(),
                [
                    table::cell(PROVIDER_COLUMNS[0], typo::body(provider.clone())),
                    table::cell(PROVIDER_COLUMNS[1], typo::body(calls.to_string())),
                    table::cell(
                        PROVIDER_COLUMNS[2],
                        typo::caption(format!("{} %", format::percent(*calls, llm.total))),
                    ),
                ],
            ));
        }
        rows.into()
    };

    surface::panel(
        column![
            surface::section_header(
                "Mes CV passent-ils les filtres ?",
                badge::count(app.data.ats_scores.len()),
            ),
            surface::divider(),
            container(meter::ats(ats_average)).padding([space::LG, 0.0]),
            surface::divider(),
            container(
                column![
                    inspector::property("Appels IA", llm.total.to_string()),
                    inspector::property("Taux de succès", format!("{} %", llm.success_rate())),
                    inspector::property(
                        "Latence moyenne",
                        format!("{} ms", llm.average_latency_ms),
                    ),
                ]
                .spacing(0),
            )
            .padding([space::SM, 0.0]),
            surface::divider(),
            table::header(app.layout(), &PROVIDER_COLUMNS),
            providers,
        ]
        .height(Length::Fill),
    )
    .width(Length::FillPortion(2))
    .height(Length::Fill)
    .into()
}
