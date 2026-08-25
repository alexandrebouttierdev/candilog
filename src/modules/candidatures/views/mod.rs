//! Écran Candidatures : en-tête de page, bande de pilotage, filtres, pipeline et vue Liste.

pub mod filters;
pub mod inspector;
pub mod list;
pub mod pipeline;

use crate::app::message::CandidateView;
use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::ui::components::button as controls;
use crate::ui::components::choice::Choice;
use crate::ui::components::header;
use crate::ui::components::icon::Icon;
use crate::ui::components::{badge, layout, pagination, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Rend l'écran complet des candidatures.
pub fn view(app: &App) -> Element<'_, Message> {
    let candidates = app.sorted_candidates();
    let companies = company_choices(app);

    let actions = row![
        controls::ghost("Exporter", Some(Icon::Download)).on_press(Message::ExportCandidatures),
        controls::primary("Nouvelle candidature", Some(Icon::Plus))
            .on_press(Message::OpenDialog(Dialog::Candidature)),
    ]
    .spacing(space::SM)
    .align_y(Alignment::Center)
    .into();

    let filter_sheet: Element<'_, Message> = if app.filters_open {
        container(filters::sheet(app, companies))
            .padding([space::LG, 0.0])
            .into()
    } else {
        iced::widget::Space::with_height(0).into()
    };

    let body = column![
        command_bar(app),
        filter_sheet,
        container(match app.candidate_view {
            CandidateView::Kanban => pipeline::view(app, &candidates),
            CandidateView::List => list::view(app, &candidates),
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([space::LG, 0.0]),
        candidate_pagination(app),
    ]
    .spacing(0)
    .height(Length::Fill);

    layout::screen(
        header::page_header(
            Icon::Applications,
            "Suivi des candidatures",
            "Suivi de vos dossiers",
            actions,
        ),
        layout::workspace(body),
    )
}

fn candidate_pagination(app: &App) -> Element<'_, Message> {
    if app.data.candidatures_total_pages <= 1 {
        return iced::widget::Space::with_height(0).into();
    }
    let (first, last) = pagination::window(
        app.candidate_page,
        crate::app::state::BUSINESS_PAGE_SIZE,
        app.data.candidatures_total,
    );
    container(pagination::pagination(
        app.candidate_page,
        app.data.candidatures_total_pages,
        Message::CandidatePagePrev,
        Message::CandidatePageNext,
        first,
        last,
        app.data.candidatures_total,
    ))
    .padding([space::SM, space::LG])
    .into()
}

/// Barre de commandes compacte : recherche, filtres et mode d'affichage.
fn command_bar(app: &App) -> Element<'_, Message> {
    let mut leading = row![
        typo::caption(format::plural(
            usize::try_from(app.data.candidatures_total).unwrap_or(usize::MAX),
            "résultat",
            "résultats",
        )),
        controls::ghost("Filtres", Some(Icon::Filter)).on_press(Message::ToggleFilters),
        badge::count_toned(app.candidate_filters.active_count(), Tone::Accent),
    ]
    .spacing(space::SM)
    .align_y(Alignment::Center);
    if !app.search.trim().is_empty() || app.candidate_filters.active_count() > 0 {
        leading = leading.push(
            controls::ghost("Tout réinitialiser", Some(Icon::Refresh))
                .on_press(Message::ResetCandidateFilters),
        );
    }
    let toolbar = row![
        leading,
        iced::widget::Space::with_width(Length::Fill),
        controls::segmented([
            controls::segment("Kanban", app.candidate_view == CandidateView::Kanban)
                .on_press(Message::CandidateViewChanged(CandidateView::Kanban)),
            controls::segment("Liste", app.candidate_view == CandidateView::List)
                .on_press(Message::CandidateViewChanged(CandidateView::List)),
        ]),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center);

    container(toolbar)
        .height(58.0)
        .padding([space::SM, space::LG])
        .width(Length::Fill)
        .align_y(Alignment::Center)
        .style(styles::glass_card)
        .into()
}

/// Options de sélection d'entreprise, précédées d'une option « toutes ».
#[must_use]
pub fn company_choices(app: &App) -> Vec<Choice> {
    std::iter::once(Choice::any("Toutes les entreprises"))
        .chain(
            app.data
                .company_options
                .iter()
                .map(|item| Choice::new(item.id, item.nom.clone())),
        )
        .collect()
}

/// Options de sélection d'une candidature, pour les formulaires liés.
#[must_use]
pub fn candidate_choices(app: &App) -> Vec<Choice> {
    app.data
        .candidate_options
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
