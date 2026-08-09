//! Écran Réseau : contacts à deux volets liste / détail.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::contacts::components as people;
use crate::modules::contacts::model::Contact;
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon};
use crate::ui::components::{badge, field, inspector, layout, list, state, surface, toolbar, typo};
use crate::ui::format;
use crate::ui::theme::metrics::{size, space};
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Rend l'écran du réseau professionnel.
pub fn view(app: &App) -> Element<'_, Message> {
    let needle = app.search.trim().to_lowercase();
    let contacts: Vec<&Contact> = app
        .data
        .contacts
        .iter()
        .filter(|item| people::matches(item, &needle))
        .collect();

    let leading = toolbar::group([
        badge::count(contacts.len()),
        toolbar::separator(),
        field::search(
            "Rechercher un contact…",
            &app.search,
            Message::SearchChanged,
            Length::Fixed(size::SEARCH),
        ),
    ]);
    let trailing = toolbar::group([controls::primary("Nouveau contact", Some(Icon::Plus))
        .on_press(Message::OpenDialog(Dialog::Contact))
        .into()]);

    layout::screen(
        toolbar::toolbar("Réseau", leading, trailing),
        layout::split(directory(app, &contacts), detail(app)),
    )
}

fn directory<'a>(app: &App, contacts: &[&'a Contact]) -> Element<'a, Message> {
    let body: Element<'a, Message> = if contacts.is_empty() {
        state::empty(
            "Aucun contact",
            "Ajoutez un contact ou modifiez votre recherche.",
        )
    } else {
        let mut rows = column![];
        for contact in contacts {
            rows = rows.push(list::row_item(
                people::full_name(contact),
                people::subtitle(contact),
                iced::widget::Space::with_width(0),
                app.selected_contact == Some(contact.id),
                Message::SelectContact(Some(contact.id)),
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
    let Some(contact) = app.focused_contact() else {
        return state::no_selection("Sélectionnez un contact pour afficher sa fiche.");
    };

    let company = contact
        .entreprise_id
        .and_then(|id| app.data.entreprises.iter().find(|item| item.id == id));
    let interviews: Vec<_> = app
        .data
        .entretiens
        .iter()
        .filter(|item| item.contact_id == Some(contact.id))
        .collect();
    let candidatures: Vec<_> = app
        .data
        .candidatures
        .iter()
        .filter(|item| item.contact_id == Some(contact.id))
        .collect();

    let header = container(
        row![
            column![
                typo::title(format!("{} {}", contact.prenom, contact.nom)),
                typo::meta(format::or_else(
                    company.map(|item| item.nom.as_str()),
                    "Entreprise non rattachée"
                )),
            ]
            .spacing(1),
            layout::spacer(),
            controls::ghost("Modifier", Some(Icon::Edit))
                .on_press(Message::EditContact(contact.id)),
            controls::icon_danger(
                Icon::Trash,
                "Supprimer",
                Message::OpenDialog(Dialog::DeleteContact(contact.id)),
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
        state::empty_slot("Aucune candidature liée.")
    } else {
        linked.into()
    };

    let mut agenda = column![].spacing(0);
    for interview in interviews.iter().take(8) {
        agenda = agenda.push(list::row_static(
            icon::muted(Icon::Calendar),
            typo::body(interview.type_entretien.to_string()),
            typo::caption(format::compact_datetime(&interview.date_entretien)),
        ));
    }
    let agenda_block: Element<'_, Message> = if interviews.is_empty() {
        state::empty_slot("Aucun entretien planifié avec ce contact.")
    } else {
        agenda.into()
    };

    let body = column![
        inspector::group(
            "Coordonnées",
            [
                inspector::property("Poste", format::or_dash(contact.poste.as_deref())),
                inspector::property("E-mail", format::or_dash(contact.email.as_deref())),
                inspector::property("Téléphone", format::or_dash(contact.telephone.as_deref())),
                inspector::property("LinkedIn", format::or_dash(contact.linkedin.as_deref())),
            ],
        ),
        column![
            row![
                typo::label("Candidatures"),
                layout::spacer(),
                badge::count(candidatures.len()),
            ]
            .align_y(Alignment::Center),
            surface::divider(),
            linked_block,
        ]
        .spacing(space::XS),
        column![
            row![
                typo::label("Entretiens"),
                layout::spacer(),
                badge::count(interviews.len()),
            ]
            .align_y(Alignment::Center),
            surface::divider(),
            agenda_block,
        ]
        .spacing(space::XS),
        inspector::note("Notes", contact.notes.clone()),
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
