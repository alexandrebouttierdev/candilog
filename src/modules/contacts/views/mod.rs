//! Écran Réseau : métriques, grille de cartes contacts, fiche en drawer.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::contacts::components as people;
use crate::modules::contacts::model::Contact;
use crate::modules::entretiens::model::Entretien;
use crate::navigation::Route;
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::Icon;
use crate::ui::components::overlay;
use crate::ui::components::{badge, field, layout, pagination, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use iced::widget::{column, container, row, stack};

pub mod drawer;

use drawer::drawer_content;
use iced::{Alignment, Element, Length};

/// Rend l'écran du réseau professionnel.
pub fn view(app: &App) -> Element<'_, Message> {
    let screen: Element<'_, Message> = layout::screen(
        header::route_header(
            Icon::Network,
            "Réseau professionnel",
            Route::Reseau,
            Message::Navigate,
            actions(),
        ),
        layout::workspace(directory(app)),
    );

    if app.focused_contact().is_some() {
        stack(vec![
            screen,
            overlay::drawer(drawer_content(app), Message::CloseContactCard),
        ])
        .into()
    } else {
        screen
    }
}

/// Actions de l'en-tête : créer un contact.
fn actions() -> Element<'static, Message> {
    controls::primary("Nouveau contact", Some(Icon::Plus))
        .on_press(Message::OpenDialog(Dialog::Contact))
        .into()
}

/// Panneau de recherche puis grille de cartes contacts.
fn directory(app: &App) -> Element<'_, Message> {
    let needle = app.search.trim().to_lowercase();
    let contacts: Vec<&Contact> = app
        .data
        .contacts
        .iter()
        .filter(|item| people::matches(item, &needle))
        .collect();

    let body: Element<'_, Message> = if contacts.is_empty() {
        state::empty(
            "Aucun contact",
            "Ajoutez un contact ou modifiez votre recherche.",
        )
    } else {
        let mut grid = row![].spacing(space::MD);
        for contact in contacts {
            grid = grid.push(
                container(people::contact_card(
                    contact,
                    Message::SelectContact(Some(contact.id)),
                ))
                .width(Length::FillPortion(1))
                .max_width(280.0),
            );
        }
        surface::scroll(container(grid.wrap()).padding(space::XL))
            .height(Length::Fill)
            .into()
    };

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let toolbar = container(
        column![
            row![
                typo::label("Votre réseau"),
                layout::spacer(),
                typo::caption(format!(
                    "{} candidatures liées · {} entretiens planifiés",
                    app.data.candidature_stats.linked_contacts,
                    entretiens_planifies(&app.data.entretiens, &today),
                )),
            ]
            .align_y(Alignment::Center),
            row![
                field::search_resettable(
                    "Rechercher un contact…",
                    &app.search,
                    Message::SearchChanged,
                    Message::ResetSearch,
                    Length::Fixed(420.0),
                ),
                badge::badge(
                    format::plural(
                        usize::try_from(app.data.contacts_total).unwrap_or(usize::MAX),
                        "contact",
                        "contacts",
                    ),
                    crate::ui::theme::Tone::Neutral,
                ),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
        ]
        .spacing(space::MD),
    )
    .height(82.0)
    .padding([space::MD, space::XL])
    .width(Length::Fill);

    let footer: Element<'_, Message> = if app.data.contacts_total_pages > 1 {
        let (first, last) = pagination::window(
            app.contact_page,
            crate::app::state::BUSINESS_PAGE_SIZE,
            app.data.contacts_total,
        );
        container(pagination::pagination(
            app.contact_page,
            app.data.contacts_total_pages,
            Message::ContactPagePrev,
            Message::ContactPageNext,
            first,
            last,
            app.data.contacts_total,
        ))
        .padding(space::MD)
        .into()
    } else {
        iced::widget::Space::with_height(0).into()
    };

    container(column![toolbar, surface::divider(), body, footer].height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(styles::panel_flat)
        .into()
}

/// Nombre total de candidatures liées à au moins un contact.
#[must_use]
#[cfg(test)]
fn total_candidatures_liees(
    candidates: &[crate::modules::candidatures::model::Candidature],
) -> usize {
    candidates
        .iter()
        .filter(|item| item.contact_id.is_some())
        .count()
}

/// Nombre d'entretiens planifiés à partir de la date donnée (tous contacts).
#[must_use]
fn entretiens_planifies(interviews: &[Entretien], today: &str) -> usize {
    interviews
        .iter()
        .filter(|item| item.date_entretien.as_str() >= today)
        .count()
}

#[cfg(test)]
#[path = "tests/mod/mod.rs"]
mod tests;
