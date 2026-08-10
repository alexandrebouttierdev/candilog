//! Écran Candidatures : barre d'outils, filtres, pipeline et vue Liste.

pub mod filters;
pub mod inspector;
pub mod list;
pub mod pipeline;

use crate::app::message::CandidateView;
use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::ui::components::button as controls;
use crate::ui::components::choice::Choice;
use crate::ui::components::icon::Icon;
use crate::ui::components::{badge, field, layout, toolbar, typo};
use crate::ui::format;
use crate::ui::theme::metrics::{size, space};
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Rend l'écran complet des candidatures.
pub fn view(app: &App) -> Element<'_, Message> {
    let candidates = app.sorted_candidates();
    let active_filters = app.candidate_filters.active_count();
    let companies = company_choices(app);
    let labels = filters::active_labels(&app.candidate_filters, &companies);

    let leading = toolbar::group([
        badge::count(candidates.len()),
        toolbar::separator(),
        field::search(
            "Rechercher un poste, une entreprise…",
            &app.search,
            Message::SearchChanged,
            Length::Fixed(size::SEARCH),
        ),
        filters_button(active_filters, app.filters_open),
        controls::segmented([
            controls::segment("Pipeline", app.candidate_view == CandidateView::Kanban)
                .on_press(Message::CandidateViewChanged(CandidateView::Kanban)),
            controls::segment("Liste", app.candidate_view == CandidateView::List)
                .on_press(Message::CandidateViewChanged(CandidateView::List)),
        ]),
    ]);

    let trailing = toolbar::group([
        toolbar::action(
            app.layout(),
            "Exporter",
            Some(Icon::Download),
            Message::ExportCandidatures,
        ),
        toolbar::primary_action(
            app.layout(),
            "Nouvelle candidature",
            Some(Icon::Plus),
            Message::OpenDialog(Dialog::Candidature),
        ),
    ]);

    let body = column![
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
    .height(Length::Fill);

    let bar = toolbar::toolbar("Candidatures", leading, trailing);
    if labels.is_empty() {
        layout::screen(bar, body)
    } else {
        layout::screen_with_strip(bar, toolbar::strip(filters::active_strip(&labels)), body)
    }
}

fn filters_button<'a>(active: usize, open: bool) -> Element<'a, Message> {
    let label: Element<'a, Message> = if active == 0 {
        typo::body("Filtres").into()
    } else {
        row![
            typo::body("Filtres"),
            badge::badge(active.to_string(), Tone::Accent),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center)
        .into()
    };
    iced::widget::button(
        row![
            crate::ui::components::icon::icon(
                Icon::Filter,
                crate::ui::components::icon::SM,
                if open || active > 0 {
                    crate::ui::components::icon::Ink::Accent
                } else {
                    crate::ui::components::icon::Ink::Muted
                },
            ),
            label,
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
    )
    .height(size::CONTROL)
    .padding([0.0, 8.0])
    .style(if open {
        crate::ui::theme::styles::selected
    } else {
        crate::ui::theme::styles::ghost
    })
    .on_press(Message::ToggleFilters)
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
