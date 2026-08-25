//! Écran Statistiques : onglet Candidatures (barres, donut, entonnoir,
//! relances) et onglet Performance CV (scores ATS et appels IA paginés).

use crate::app::state::StatisticsTab;
use crate::app::{App, Message};
use crate::modules::metriques::components::PipelineCounts;
use crate::ui::components::icon::Icon;
use crate::ui::components::tabs::Tab;
use crate::ui::components::{header, layout, tabs};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use iced::widget::{column, container};
use iced::{Element, Length};

pub mod candidatures;
pub mod performance;

use candidatures::candidatures_tab;
use performance::performance_tab;

/// Nombre de lignes par page des historiques ATS et IA.
pub const PAGE_SIZE: u64 = 10;

/// Colonnes de l'historique des scores ATS.
/// Colonnes de l'historique des appels IA.
/// Rend l'écran des statistiques.
pub fn view(app: &App) -> Element<'_, Message> {
    let counts = PipelineCounts::from_stats(&app.data.candidature_stats);
    let section_tabs = tabs::segmented(
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
        |index| {
            Message::StatisticsTabChanged(if index == 0 {
                StatisticsTab::Candidatures
            } else {
                StatisticsTab::PerformanceCv
            })
        },
    );
    layout::screen(
        header::page_header(
            Icon::Chart,
            "Analyses et performance",
            format::plural(counts.total, "candidature suivie", "candidatures suivies"),
            iced::widget::Space::with_width(0).into(),
        ),
        layout::workspace(
            column![
                container(section_tabs).padding([space::SM, space::LG]),
                match app.statistics_tab {
                    StatisticsTab::Candidatures => candidatures_tab(app, &counts),
                    StatisticsTab::PerformanceCv => performance_tab(app),
                },
            ]
            .spacing(space::SM)
            .width(Length::Fill),
        ),
    )
}
