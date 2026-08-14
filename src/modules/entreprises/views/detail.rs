//! Fiche de l'entreprise sélectionnée : coordonnées, candidatures et contacts liés.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::ui::components::avatar;
use crate::ui::components::button as controls;
use crate::ui::components::icon::Icon;
use crate::ui::components::stat_card;
use crate::ui::components::{badge, inspector, layout, list, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Fiche de l'entreprise sélectionnée.
pub fn detail(app: &App) -> Element<'_, Message> {
    let Some(company) = app.focused_company() else {
        return container(state::no_selection(
            "Sélectionnez une entreprise pour afficher sa fiche.",
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(styles::panel_flat)
        .into();
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
            avatar::avatar(avatar::initials_of(&company.nom), 48.0, Tone::Accent),
            column![
                typo::title(company.nom.clone()),
                typo::meta(format::or_else(
                    company.secteur.as_deref(),
                    "Secteur non renseigné"
                )),
            ]
            .spacing(1),
            layout::spacer(),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .padding(space::XL)
    .style(styles::sunken);

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
        let name = format!("{} {}", contact.prenom, contact.nom);
        people = people.push(list::row_static(
            avatar::avatar(avatar::initials_of(&name), 26.0, Tone::Neutral),
            typo::body(name),
            typo::caption(format::or_dash(contact.poste.as_deref())),
        ));
    }
    let people_block: Element<'_, Message> = if contacts.is_empty() {
        state::empty_slot("Aucun contact rattaché.")
    } else {
        people.into()
    };

    let body = column![
        row![
            stat_card::metric_icon_tinted(
                "Candidatures",
                candidatures.len().to_string(),
                Tone::Accent,
                Icon::Applications,
            ),
            stat_card::metric_icon_tinted(
                "Contacts",
                contacts.len().to_string(),
                Tone::Neutral,
                Icon::Network,
            ),
        ]
        .spacing(space::MD),
        inspector::group(
            "Coordonnées",
            [
                inspector::property("Site web", format::or_dash(company.site_web.as_deref())),
                inspector::property("Ville", format::or_dash(company.ville.as_deref())),
                inspector::property("Adresse", format::or_dash(company.adresse.as_deref())),
            ],
        ),
        column![
            row![
                typo::label("Candidatures liées"),
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
                typo::label("Contacts liés"),
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

    container(
        column![
            header,
            surface::divider(),
            surface::scroll(container(body).padding([space::XL, 0.0])).height(Length::Fill),
            surface::divider(),
            container(inspector::actions([
                controls::secondary("Modifier", Some(Icon::Edit))
                    .on_press(Message::EditEntreprise(company.id))
                    .into(),
                controls::danger("Supprimer", Some(Icon::Trash))
                    .on_press(Message::OpenDialog(Dialog::DeleteEntreprise(company.id)))
                    .into(),
            ]))
            .padding([space::LG, space::XL]),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(styles::panel_flat)
    .into()
}
