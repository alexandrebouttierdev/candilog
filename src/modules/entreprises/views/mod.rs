//! Écran Entreprises : métriques, répertoire et fiche d'entreprise.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::candidatures::model::Candidature;
use crate::modules::entreprises::components as directory_entry;
use crate::modules::entreprises::model::Entreprise;
use crate::ui::components::avatar;
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::Icon;
use crate::ui::components::stat_card;
use crate::ui::components::{badge, field, inspector, layout, list, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Rend l'écran des entreprises.
pub fn view(app: &App) -> Element<'_, Message> {
    let metrics = row![
        stat_card::metric_icon_tinted(
            "Entreprises suivies",
            app.data.entreprises.len().to_string(),
            Tone::Accent,
            Icon::Building,
        ),
        stat_card::metric_icon_tinted(
            "Candidatures liées",
            total_candidatures(&app.data.candidatures, app.selected_company).to_string(),
            Tone::Info,
            Icon::Applications,
        ),
        stat_card::metric_icon_tinted(
            "Contacts enregistrés",
            app.data.contacts.len().to_string(),
            Tone::Success,
            Icon::Profile,
        ),
    ]
    .spacing(space::MD);

    let actions = controls::primary("Nouvelle entreprise", Some(Icon::Plus))
        .on_press(Message::OpenDialog(Dialog::Entreprise))
        .into();

    layout::screen(
        header::page_header(
            Icon::Building,
            "Entreprises",
            "Votre répertoire professionnel",
            actions,
        ),
        layout::workspace(
            column![metrics, layout::columns([directory(app), detail(app)]),]
                .spacing(space::LG)
                .height(Length::Fill),
        ),
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

    container(
        column![
            container(field::search(
                "Rechercher une entreprise…",
                &app.search,
                Message::SearchChanged,
                Length::Fill,
            ))
            .padding(space::LG),
            surface::divider(),
            body,
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(styles::glass_card)
    .into()
}

/// Fiche de l'entreprise sélectionnée.
fn detail(app: &App) -> Element<'_, Message> {
    let Some(company) = app.focused_company() else {
        return container(state::no_selection(
            "Sélectionnez une entreprise pour afficher sa fiche.",
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(styles::glass_card)
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
                controls::ghost("Modifier", Some(Icon::Edit))
                    .on_press(Message::EditEntreprise(company.id))
                    .into(),
                controls::icon_danger(
                    Icon::Trash,
                    "Supprimer",
                    Message::OpenDialog(Dialog::DeleteEntreprise(company.id)),
                ),
            ]))
            .padding([space::LG, space::XL]),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(styles::glass_card)
    .into()
}

/// Nombre de candidatures liées à l'entreprise sélectionnée.
#[must_use]
fn total_candidatures(candidates: &[Candidature], selected: Option<uuid::Uuid>) -> usize {
    let Some(id) = selected else {
        return 0;
    };
    candidates
        .iter()
        .filter(|item| item.entreprise_id == id)
        .count()
}

#[cfg(test)]
mod tests {
    use super::total_candidatures;
    use crate::modules::candidatures::model::{Candidature, StatutCandidature, TypeContrat};
    use uuid::Uuid;

    fn candidature(entreprise_id: Uuid) -> Candidature {
        Candidature {
            id: Uuid::new_v4(),
            poste: "Développeur".into(),
            entreprise_id,
            entreprise_nom: Some("Agrial".into()),
            contact_id: None,
            type_contrat: TypeContrat::Cdi,
            statut: StatutCandidature::EnAttente,
            date_envoi: "2026-08-01".into(),
            lien_offre: None,
            notes: None,
            created_at: "2026-08-01".into(),
            updated_at: "2026-08-01".into(),
        }
    }

    #[test]
    fn sans_selection_aucune_candidature_n_est_comptee() {
        assert_eq!(total_candidatures(&[], None), 0);
    }

    #[test]
    fn ne_comptent_que_les_candidatures_de_l_entreprise_selectionnee() {
        let target = Uuid::new_v4();
        let other = Uuid::new_v4();
        let candidates = vec![candidature(target), candidature(target), candidature(other)];
        assert_eq!(total_candidatures(&candidates, Some(target)), 2);
        assert_eq!(total_candidatures(&candidates, Some(other)), 1);
    }
}
