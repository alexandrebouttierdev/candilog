//! Écran Entreprises : répertoire à deux volets liste / détail.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::entreprises::components as directory_entry;
use crate::modules::entreprises::model::Entreprise;
use crate::ui::components::button as controls;
use crate::ui::components::icon::Icon;
use crate::ui::components::{badge, field, inspector, layout, list, state, surface, toolbar, typo};
use crate::ui::format;
use crate::ui::theme::metrics::{size, space};
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Rend l'écran des entreprises.
pub fn view(app: &App) -> Element<'_, Message> {
    let needle = app.search.trim().to_lowercase();
    let companies: Vec<&Entreprise> = app
        .data
        .entreprises
        .iter()
        .filter(|item| directory_entry::matches(item, &needle))
        .collect();

    let leading = toolbar::group([
        badge::count(companies.len()),
        toolbar::separator(),
        field::search(
            "Rechercher une entreprise…",
            &app.search,
            Message::SearchChanged,
            Length::Fixed(size::SEARCH),
        ),
    ]);
    let trailing = toolbar::group([controls::primary("Nouvelle entreprise", Some(Icon::Plus))
        .on_press(Message::OpenDialog(Dialog::Entreprise))
        .into()]);

    layout::screen(
        toolbar::toolbar("Entreprises", leading, trailing),
        layout::split(directory(app, &companies), detail(app)),
    )
}

fn directory<'a>(app: &App, companies: &[&'a Entreprise]) -> Element<'a, Message> {
    let body: Element<'a, Message> = if companies.is_empty() {
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
                company.nom.clone(),
                directory_entry::subtitle(company),
                badge::count(count),
                app.selected_company == Some(company.id),
                Message::SelectCompany(Some(company.id)),
            ));
        }
        surface::scroll(rows).height(Length::Fill).into()
    };
    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(crate::ui::theme::styles::panel_flat)
        .into()
}

fn detail(app: &App) -> Element<'_, Message> {
    let Some(company) = app.focused_company() else {
        return state::no_selection("Sélectionnez une entreprise pour afficher sa fiche.");
    };

    let candidatures: Vec<_> = app
        .data
        .candidatures
        .iter()
        .filter(|item| item.entreprise_id == company.id)
        .collect();
    let contacts: Vec<_> = app
        .data
        .contacts
        .iter()
        .filter(|item| item.entreprise_id == Some(company.id))
        .collect();

    let header = container(
        row![
            column![
                typo::title(company.nom.clone()),
                typo::meta(format::or_else(
                    company.secteur.as_deref(),
                    "Secteur non renseigné"
                )),
            ]
            .spacing(1),
            layout::spacer(),
            controls::ghost("Modifier", Some(Icon::Edit))
                .on_press(Message::EditEntreprise(company.id)),
            controls::icon_danger(
                Icon::Trash,
                "Supprimer",
                Message::OpenDialog(Dialog::DeleteEntreprise(company.id)),
            ),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
    )
    .padding(space::XL);

    let mut linked = column![].spacing(0);
    for candidature in candidatures.iter().take(8) {
        linked = linked.push(list::row_static(
            crate::modules::candidatures::components::glyph(candidature.statut),
            typo::body(format::truncate(&candidature.poste, 40)),
            typo::caption(format::compact_date(&candidature.date_envoi)),
        ));
    }
    let linked_block: Element<'_, Message> = if candidatures.is_empty() {
        state::empty_slot("Aucune candidature envoyée à cette entreprise.")
    } else {
        linked.into()
    };

    let mut people = column![].spacing(0);
    for contact in contacts.iter().take(8) {
        people = people.push(list::row_static(
            crate::ui::components::icon::muted(Icon::Profile),
            typo::body(format!("{} {}", contact.prenom, contact.nom)),
            typo::caption(format::or_dash(contact.poste.as_deref())),
        ));
    }
    let people_block: Element<'_, Message> = if contacts.is_empty() {
        state::empty_slot("Aucun contact rattaché.")
    } else {
        people.into()
    };

    let body = column![
        inspector::group(
            "Coordonnées",
            [
                inspector::property("Ville", format::or_dash(company.ville.as_deref())),
                inspector::property("Adresse", format::or_dash(company.adresse.as_deref())),
                inspector::property("Site web", format::or_dash(company.site_web.as_deref())),
                inspector::property("Type", format::or_dash(company.type_.as_deref())),
            ],
        ),
        column![
            row![
                typo::label("Candidatures"),
                layout::spacer(),
                badge::count_toned(candidatures.len(), Tone::Accent),
            ]
            .align_y(Alignment::Center),
            surface::divider(),
            linked_block,
        ]
        .spacing(space::XS),
        column![
            row![
                typo::label("Contacts"),
                layout::spacer(),
                badge::count(contacts.len()),
            ]
            .align_y(Alignment::Center),
            surface::divider(),
            people_block,
        ]
        .spacing(space::XS),
        inspector::note("Notes", company.notes.clone()),
    ]
    .spacing(space::XXL)
    .padding([0.0, space::XL]);

    column![
        header,
        surface::divider(),
        surface::scroll(container(body).padding([space::XL, 0.0])).height(Length::Fill),
    ]
    .height(Length::Fill)
    .into()
}
