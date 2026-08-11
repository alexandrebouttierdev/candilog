//! Inspecteur d'une candidature : propriétés alignées, activité, actions.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::candidatures::components::{activity_row, status_tone, PIPELINE};
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon};
use crate::ui::components::{field, inspector, layout, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Contenu du drawer d'inspection d'une candidature.
pub fn view(app: &App, id: uuid::Uuid) -> Element<'_, Message> {
    let Some(candidate) = app.data.candidatures.iter().find(|item| item.id == id) else {
        return state::no_selection("Cette candidature n'existe plus.");
    };
    let company = app
        .data
        .entreprises
        .iter()
        .find(|item| item.id == candidate.entreprise_id);
    let contact = candidate
        .contact_id
        .and_then(|contact_id| app.data.contacts.iter().find(|item| item.id == contact_id));
    let interviews: Vec<_> = app
        .data
        .entretiens
        .iter()
        .filter(|item| item.candidature_id == id)
        .collect();
    let reminders: Vec<_> = app
        .data
        .relances
        .iter()
        .filter(|item| item.candidature_id == id)
        .collect();

    let header = container(
        column![
            row![
                container(icon::toned(
                    Icon::Applications,
                    status_tone(candidate.statut),
                ))
                .width(44.0)
                .height(44.0)
                .center(Length::Fixed(44.0))
                .style(styles::toned(status_tone(candidate.statut))),
                column![
                    typo::title(candidate.poste.clone()),
                    typo::meta(format::or_else(
                        candidate.entreprise_nom.as_deref(),
                        "Entreprise inconnue"
                    )),
                ]
                .spacing(1),
                layout::spacer(),
                controls::icon_action(Icon::Close, "Fermer", Message::CloseDialog),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
            container(
                row![
                    column![
                        typo::label("Statut de la candidature"),
                        typo::caption("Choisissez directement l'étape du suivi"),
                    ]
                    .spacing(1),
                    layout::spacer(),
                    field::select(PIPELINE.to_vec(), Some(candidate.statut), move |target| {
                        Message::MoveCandidature(id, target)
                    },)
                    .width(Length::Fixed(188.0)),
                ]
                .spacing(space::MD)
                .align_y(Alignment::Center),
            )
            .padding([space::SM, space::MD])
            .width(Length::Fill)
            .style(styles::sunken),
        ]
        .spacing(space::MD),
    )
    .padding(space::XL);

    let mut activity = column![].spacing(0);
    for interview in &interviews {
        activity = activity.push(activity_row(
            "Entretien",
            format!(
                "{} · {}",
                format::compact_datetime(&interview.date_entretien),
                interview.type_entretien
            ),
            Tone::Success,
        ));
    }
    for reminder in &reminders {
        activity = activity.push(activity_row(
            "Relance",
            format!(
                "{} · {}",
                format::compact_date(&reminder.date_relance),
                reminder.type_relance
            ),
            Tone::Warning,
        ));
    }
    let activity_block: Element<'_, Message> = if interviews.is_empty() && reminders.is_empty() {
        state::empty_slot("Aucun entretien ni relance rattaché.")
    } else {
        activity.into()
    };

    let body = column![
        inspector::group(
            "Candidature",
            [
                inspector::property("Contrat", candidate.type_contrat.to_string()),
                inspector::property("Envoyée le", format::compact_date(&candidate.date_envoi)),
                inspector::property("Offre", format::or_dash(candidate.lien_offre.as_deref()),),
            ],
        ),
        inspector::group(
            "Entreprise",
            [
                inspector::property(
                    "Nom",
                    format::or_else(company.map(|item| item.nom.as_str()), "Inconnue"),
                ),
                inspector::property(
                    "Secteur",
                    format::or_dash(company.and_then(|item| item.secteur.as_deref())),
                ),
                inspector::property(
                    "Ville",
                    format::or_dash(company.and_then(|item| item.ville.as_deref())),
                ),
            ],
        ),
        inspector::group(
            "Contact",
            [
                inspector::property(
                    "Interlocuteur",
                    contact.map_or_else(
                        || "Aucun".to_owned(),
                        |item| format!("{} {}", item.prenom, item.nom),
                    ),
                ),
                inspector::property(
                    "E-mail",
                    format::or_dash(contact.and_then(|item| item.email.as_deref())),
                ),
                inspector::property(
                    "Téléphone",
                    format::or_dash(contact.and_then(|item| item.telephone.as_deref())),
                ),
            ],
        ),
        column![typo::label("Activité"), surface::divider(), activity_block,].spacing(space::XS),
        inspector::note("Notes", candidate.notes.clone()),
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
                    .on_press(Message::OpenDialog(Dialog::DeleteCandidature(id))),
                layout::spacer(),
                controls::secondary("Modifier", Some(Icon::Edit))
                    .on_press(Message::EditCandidature(id)),
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
