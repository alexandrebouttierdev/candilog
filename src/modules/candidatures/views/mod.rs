//! Écran Candidatures : en-tête de page, bande de pilotage, filtres, pipeline et vue Liste.

pub mod filters;
pub mod inspector;
pub mod list;
pub mod pipeline;

use crate::app::message::CandidateView;
use crate::app::{App, Message};
use crate::modules::metriques::components::PipelineCounts;
use crate::ui::components::button as controls;
use crate::ui::components::choice::Choice;
use crate::ui::components::header;
use crate::ui::components::icon::Icon;
use crate::ui::components::{badge, field, layout, typo};
use crate::ui::format;
use crate::ui::theme::metrics::{size, space};
use crate::ui::theme::styles;
use crate::ui::theme::typography::MONO_SEMIBOLD;
use crate::ui::theme::{Marker, Tone};
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Rend l'écran complet des candidatures.
pub fn view(app: &App) -> Element<'_, Message> {
    let candidates = app.sorted_candidates();
    let companies = company_choices(app);

    let meta = format::plural(
        app.filtered_candidates().len(),
        "candidature suivie",
        "candidatures suivies",
    );
    let actions = controls::ghost("Exporter", Some(Icon::Download))
        .on_press(Message::ExportCandidatures)
        .into();

    let body = column![
        control_strip(app),
        if app.filters_open {
            filters::sheet(app, companies)
        } else {
            iced::widget::Space::with_height(0).into()
        },
        container(match app.candidate_view {
            CandidateView::Kanban => pipeline::view(app, &candidates),
            CandidateView::List => list::view(app, &candidates),
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([space::XL, space::XXL - 2.0]),
    ]
    .spacing(space::LG)
    .height(Length::Fill);

    layout::screen(
        header::page_header(Icon::Applications, "Candidatures", meta, actions),
        layout::workspace(body),
    )
}

/// Bande de pilotage : 4 mini-statuts puis la toolbar de recherche et de bascule.
fn control_strip(app: &App) -> Element<'_, Message> {
    let counts = PipelineCounts::from_candidates(&app.data.candidatures);

    let statuses = row![
        mini_status("En attente", counts.pending, Tone::Info),
        mini_status("Relancées", counts.followed_up, Tone::Warning),
        mini_status("Entretien", counts.interviews, Tone::Violet),
        mini_status("Refus", counts.rejected, Tone::Danger),
    ]
    .spacing(space::MD);

    let toolbar = row![
        field::search(
            "Rechercher un poste, une entreprise…",
            &app.search,
            Message::SearchChanged,
            Length::Fixed(size::SEARCH),
        ),
        typo::caption(format::plural(
            app.filtered_candidates().len(),
            "résultat",
            "résultats",
        )),
        controls::ghost("Filtres", None).on_press(Message::ToggleFilters),
        badge::count_toned(app.candidate_filters.active_count(), Tone::Accent),
        iced::widget::Space::with_width(Length::Fill),
        controls::segmented([
            controls::segment("Kanban", app.candidate_view == CandidateView::Kanban)
                .on_press(Message::CandidateViewChanged(CandidateView::Kanban)),
            controls::segment("Liste", app.candidate_view == CandidateView::List)
                .on_press(Message::CandidateViewChanged(CandidateView::List)),
        ]),
    ]
    .spacing(space::SM)
    .align_y(Alignment::Center);

    container(
        column![statuses, toolbar]
            .spacing(space::MD)
            .padding(space::LG),
    )
    .width(Length::Fill)
    .style(styles::glass_card)
    .into()
}

/// Carte mini-statut : pastille de statut, libellé et compteur mono.
fn mini_status<'a>(label: &'a str, count: usize, tone: Tone) -> Element<'a, Message> {
    container(
        row![
            badge::marker(tone, Marker::Solid),
            column![
                typo::caption(label),
                typo::text_mono(count.to_string(), 20.0, MONO_SEMIBOLD),
            ]
            .spacing(0),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .padding(space::LG)
    .width(Length::FillPortion(1))
    .style(styles::glass_card)
    .into()
}

/// Options de sélection d'entreprise, précédées d'une option « toutes ».
#[must_use]
pub fn company_choices(app: &App) -> Vec<Choice> {
    std::iter::once(Choice::any("Toutes les entreprises"))
        .chain(
            app.data
                .entreprises
                .iter()
                .map(|item| Choice::new(item.id, item.nom.clone())),
        )
        .collect()
}

/// Options de sélection d'une candidature, pour les formulaires liés.
#[must_use]
pub fn candidate_choices(app: &App) -> Vec<Choice> {
    app.data
        .candidatures
        .iter()
        .map(|item| {
            Choice::new(
                item.id,
                format!(
                    "{} · {}",
                    format::truncate(&item.poste, 40),
                    format::or_else(item.entreprise_nom.as_deref(), "Entreprise")
                ),
            )
        })
        .collect()
}
