//! Fiche contact en drawer : coordonnées, candidatures et entretiens liés.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::contacts::components as people;
use crate::ui::components::avatar;
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon};
use crate::ui::components::{inspector, layout, list, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Contenu du drawer de la fiche contact.
pub fn drawer_content(app: &App) -> Element<'_, Message> {
    let Some(contact) = app.focused_contact() else {
        return state::no_selection("Ce contact n'existe plus.");
    };

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
            avatar::avatar(
                avatar::initials_of(&people::full_name(contact)),
                48.0,
                Tone::Accent,
            ),
            column![
                typo::title(people::full_name(contact)),
                typo::meta(format::or_else(
                    contact.poste.as_deref(),
                    "Poste non renseigné"
                )),
            ]
            .spacing(1),
            layout::spacer(),
            controls::icon_action(Icon::Close, "Fermer", Message::CloseContactCard),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .padding(space::XL);

    let linked_rows: Vec<Element<'_, Message>> = if candidatures.is_empty() {
        vec![state::empty_slot("Aucune candidature liée.")]
    } else {
        candidatures
            .iter()
            .take(8)
            .map(|candidature| {
                list::row_static(
                    crate::modules::candidatures::components::glyph(candidature.statut),
                    typo::body(format::truncate(&candidature.poste, 40)),
                    typo::caption(format::compact_date(&candidature.date_envoi)),
                )
            })
            .collect()
    };

    let agenda_rows: Vec<Element<'_, Message>> = if interviews.is_empty() {
        vec![state::empty_slot(
            "Aucun entretien planifié avec ce contact.",
        )]
    } else {
        interviews
            .iter()
            .take(8)
            .map(|interview| {
                list::row_static(
                    icon::muted(Icon::Calendar),
                    typo::body(interview.type_entretien.to_string()),
                    typo::caption(format::compact_datetime(&interview.date_entretien)),
                )
            })
            .collect()
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
        inspector::group("Candidatures liées", linked_rows),
        inspector::group("Entretiens planifiés", agenda_rows),
        inspector::note("Notes", contact.notes.clone()),
    ]
    .spacing(space::XL)
    .padding([0.0, space::XL]);

    column![
        header,
        surface::divider(),
        surface::scroll(container(body).padding([space::XL, 0.0])).height(Length::Fill),
        surface::divider(),
        container(
            row![
                controls::danger("Supprimer", Some(Icon::Trash))
                    .on_press(Message::OpenDialog(Dialog::DeleteContact(contact.id))),
                layout::spacer(),
                controls::secondary("Modifier", Some(Icon::Edit))
                    .on_press(Message::EditContact(contact.id)),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
        )
        .padding([space::LG, space::XL])
        .width(Length::Fill),
    ]
    .height(Length::Fill)
    .into()
}
