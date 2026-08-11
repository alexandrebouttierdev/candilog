//! Écran Réseau : métriques, grille de cartes contacts, fiche en drawer.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::contacts::components as people;
use crate::modules::contacts::model::Contact;
use crate::modules::entretiens::model::Entretien;
use crate::navigation::Route;
use crate::ui::components::avatar;
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::{self, Icon};
use crate::ui::components::overlay;
use crate::ui::components::{field, inspector, layout, list, pagination, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, stack};
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
        row![
            field::search(
                "Rechercher un contact…",
                &app.search,
                Message::SearchChanged,
                Length::Fixed(360.0),
            ),
            typo::caption(format::plural(
                usize::try_from(app.data.contacts_total).unwrap_or(usize::MAX),
                "contact",
                "contacts",
            )),
            layout::spacer(),
            typo::caption(format!(
                "{} candidatures liées · {} entretiens planifiés",
                app.data.candidature_stats.linked_contacts,
                entretiens_planifies(&app.data.entretiens, &today),
            )),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .height(52.0)
    .padding([0.0, space::LG])
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

/// Contenu du drawer de la fiche contact.
fn drawer_content(app: &App) -> Element<'_, Message> {
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
mod tests {
    use super::{entretiens_planifies, total_candidatures_liees};
    use crate::modules::candidatures::model::{Candidature, StatutCandidature, TypeContrat};
    use crate::modules::entretiens::model::{Entretien, TypeEntretien};
    use uuid::Uuid;

    fn candidature(contact_id: Option<Uuid>) -> Candidature {
        Candidature {
            id: Uuid::new_v4(),
            poste: "Développeur".into(),
            entreprise_id: Uuid::new_v4(),
            entreprise_nom: Some("Agrial".into()),
            contact_id,
            type_contrat: TypeContrat::Cdi,
            statut: StatutCandidature::EnAttente,
            date_envoi: "2026-08-01".into(),
            lien_offre: None,
            notes: None,
            created_at: "2026-08-01".into(),
            updated_at: "2026-08-01".into(),
        }
    }

    fn entretien(contact_id: Option<Uuid>, date: &str) -> Entretien {
        Entretien {
            id: Uuid::new_v4(),
            candidature_id: Uuid::new_v4(),
            contact_id,
            date_entretien: date.into(),
            type_entretien: TypeEntretien::Visio,
            lieu: None,
            notes: None,
            compte_rendu: None,
            analyse_ia: None,
            created_at: date.into(),
            updated_at: date.into(),
        }
    }

    #[test]
    fn les_candidatures_liees_comptent_toutes_celles_qui_ont_un_contact() {
        let target = Uuid::new_v4();
        let other = Uuid::new_v4();
        let candidates = vec![
            candidature(Some(target)),
            candidature(Some(target)),
            candidature(Some(other)),
            candidature(None),
        ];
        assert_eq!(total_candidatures_liees(&candidates), 3);
        assert_eq!(total_candidatures_liees(&[]), 0);
    }

    #[test]
    fn les_entretiens_planifies_sont_globaux_a_partir_d_aujourd_hui() {
        let interviews = vec![
            entretien(Some(Uuid::new_v4()), "2026-08-12T09:00:00"),
            entretien(Some(Uuid::new_v4()), "2026-08-08T09:00:00"),
            entretien(None, "2026-08-20T09:00:00"),
        ];
        assert_eq!(entretiens_planifies(&interviews, "2026-08-10"), 2);
        assert_eq!(entretiens_planifies(&interviews, "2026-08-13"), 1);
        assert_eq!(entretiens_planifies(&[], "2026-08-10"), 0);
    }
}
