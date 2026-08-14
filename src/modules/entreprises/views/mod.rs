//! Écran Entreprises : métriques, répertoire et fiche d'entreprise.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::entreprises::components as directory_entry;
use crate::modules::entreprises::model::Entreprise;
use crate::navigation::Route;
use crate::ui::components::avatar;
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::Icon;
use crate::ui::components::{badge, field, layout, list, pagination, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};

pub mod detail;

use detail::detail;
use iced::{Alignment, Element, Length};

/// Rend l'écran des entreprises.
pub fn view(app: &App) -> Element<'_, Message> {
    let actions = controls::primary("Nouvelle entreprise", Some(Icon::Plus))
        .on_press(Message::OpenDialog(Dialog::Entreprise))
        .into();

    layout::screen(
        header::route_header(
            Icon::Building,
            "Relations professionnelles",
            Route::Entreprises,
            Message::Navigate,
            actions,
        ),
        layout::workspace(layout::columns([directory(app), detail(app)])),
    )
}

/// Panneau du répertoire : recherche, puis liste des entreprises filtrées.
fn directory(app: &App) -> Element<'_, Message> {
    let needle = app.search.trim().to_lowercase();
    let companies: Vec<&Entreprise> = app
        .data
        .entreprises
        .iter()
        .filter(|item| directory_entry::matches(item, &needle))
        .collect();

    let body: Element<'_, Message> = if companies.is_empty() {
        state::empty(
            "Aucune entreprise",
            "Créez une entreprise ou modifiez votre recherche.",
        )
    } else {
        let mut rows = column![];
        for company in companies {
            let count = app
                .data
                .candidatures
                .iter()
                .filter(|item| item.entreprise_id == company.id)
                .count();
            rows = rows.push(list::row_item(
                avatar::avatar(avatar::initials_of(&company.nom), 36.0, Tone::Accent),
                company.nom.clone(),
                directory_entry::subtitle(company),
                badge::count(count),
                app.selected_company == Some(company.id),
                Message::SelectCompany(Some(company.id)),
            ));
        }
        surface::scroll(rows).height(Length::Fill).into()
    };

    const ALL_TYPES: &str = "Tous les types";
    let type_options: Vec<String> = std::iter::once(ALL_TYPES.to_owned())
        .chain(app.data.company_types.iter().cloned())
        .collect();
    let selected_type = Some(
        app.company_type_filter
            .clone()
            .unwrap_or_else(|| ALL_TYPES.to_owned()),
    );
    let toolbar = container(
        column![
            row![
                typo::label("Répertoire"),
                layout::spacer(),
                typo::caption(format!(
                    "{} · {} candidatures",
                    format::plural(
                        usize::try_from(app.data.entreprises_total).unwrap_or(usize::MAX),
                        "entreprise",
                        "entreprises"
                    ),
                    app.data.candidature_stats.total,
                )),
            ]
            .align_y(Alignment::Center),
            row![
                field::search_resettable(
                    "Rechercher une entreprise…",
                    &app.search,
                    Message::SearchChanged,
                    Message::ResetSearch,
                    Length::Fill,
                ),
                field::select(type_options, selected_type, |value| {
                    Message::CompanyTypeFilterChanged((value != ALL_TYPES).then_some(value))
                })
                .width(Length::Fixed(190.0)),
                controls::ghost("Réinitialiser", Some(Icon::Refresh))
                    .on_press(Message::ResetCompanyDirectory),
            ]
            .spacing(space::SM)
            .align_y(Alignment::Center),
        ]
        .spacing(space::MD),
    )
    .padding(space::LG)
    .width(Length::Fill);

    let footer: Element<'_, Message> = if app.data.entreprises_total_pages > 1 {
        let (first, last) = pagination::window(
            app.company_page,
            crate::app::state::BUSINESS_PAGE_SIZE,
            app.data.entreprises_total,
        );
        container(pagination::pagination(
            app.company_page,
            app.data.entreprises_total_pages,
            Message::CompanyPagePrev,
            Message::CompanyPageNext,
            first,
            last,
            app.data.entreprises_total,
        ))
        .padding(space::MD)
        .into()
    } else {
        iced::widget::Space::with_height(0).into()
    };

    container(column![toolbar, surface::divider(), body, footer].height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(styles::glass_card)
        .into()
}

/// Nombre de candidatures rattachées à une entreprise suivie.
///
/// Indicateur **global**, comme les deux tuiles qui l'encadrent (« Entreprises suivies »,
/// « Contacts enregistrés »). Il dépendait auparavant de la sélection et renvoyait 0 tant
/// qu'aucune entreprise n'était choisie — c'est-à-dire à l'arrivée sur l'écran : la tuile
/// affichait « Candidatures liées 0 » juste au-dessus d'une liste dont les badges totalisaient
/// 37, dans une rangée d'indicateurs par ailleurs globaux et sans distinction visuelle.
#[must_use]
#[cfg(test)]
fn total_candidatures(
    candidates: &[crate::modules::candidatures::model::Candidature],
    companies: &[Entreprise],
) -> usize {
    candidates
        .iter()
        .filter(|item| {
            companies
                .iter()
                .any(|company| company.id == item.entreprise_id)
        })
        .count()
}

#[cfg(test)]
#[path = "tests/mod/mod.rs"]
mod tests;
