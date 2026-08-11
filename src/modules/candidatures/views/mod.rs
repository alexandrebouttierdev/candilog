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
use crate::ui::components::sidebar::workspace_tab_controls;
use crate::ui::components::{badge, field, layout, typo};
use crate::ui::format;
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Background, Border, Element, Length, Theme};

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
            .padding([space::SM, space::LG])
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
        .padding([space::LG, space::LG]),
    ]
    .spacing(0)
    .height(Length::Fill);

    layout::screen(
        header::workspace_header(
            Icon::Applications,
            "Suivi des candidatures",
            workspace_tab_controls(app.route, Message::Navigate),
            actions,
        ),
        body,
    )
}

/// Barre de commandes compacte : recherche, filtres et mode d'affichage.
fn command_bar(app: &App) -> Element<'_, Message> {
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
        controls::ghost("Filtres", Some(Icon::Filter)).on_press(Message::ToggleFilters),
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

    container(toolbar)
        .height(52.0)
        .padding([space::SM, space::LG])
        .width(Length::Fill)
        .align_y(Alignment::Center)
        .style(command_bar_style)
        .into()
}

fn command_bar_style(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(palette.canvas)),
        text_color: Some(palette.text),
        border: Border {
            color: palette.border,
            width: stroke::HAIRLINE,
            radius: radius::NONE.into(),
        },
        ..container::Style::default()
    }
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
