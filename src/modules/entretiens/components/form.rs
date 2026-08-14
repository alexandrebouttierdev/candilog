//! Formulaire de création/édition d'un entretien.

use crate::app::{App, Message};
use crate::modules::candidatures::views::candidate_choices;
use crate::modules::entretiens::model::TypeEntretien;
use crate::ui::components::choice::Choice;
use crate::ui::components::field;
use crate::ui::components::icon::Icon;
use crate::ui::components::relation::{relation_page, RelationNavigation};
use crate::ui::components::typo;
use crate::ui::theme::metrics::space;
use iced::widget::column;
use iced::Element;

/// Formulaire de création/édition d'un entretien.
pub fn form(app: &App) -> Element<'_, Message> {
    let candidates = candidate_choices(app);
    let selected_candidate = Choice::find(&candidates, app.entretien_form.candidature_id);
    let contacts: Vec<Choice> = app
        .data
        .contact_options
        .iter()
        .map(|item| Choice::new(item.id, format!("{} {}", item.prenom, item.nom)))
        .collect();
    let selected_contact = Choice::find(&contacts, app.entretien_form.contact_id);
    let types = vec![
        TypeEntretien::Presentiel,
        TypeEntretien::Visio,
        TypeEntretien::Telephonique,
        TypeEntretien::Technique,
        TypeEntretien::Rh,
        TypeEntretien::Autre,
    ];

    column![
        field::form_section(
            Icon::Applications,
            "Contexte de l'entretien",
            "Candidature et interlocuteur associés",
            column![
                field::labeled(
                    "Candidature *",
                    relation_page(
                        &app.candidate_option_search,
                        Message::CandidateOptionSearchChanged,
                        crate::app::state::RELATION_PAGE_SIZE,
                        RelationNavigation {
                            page: app.candidate_option_page,
                            total_pages: app.data.candidate_options_total_pages,
                            total: app.data.candidate_options_total,
                            previous: Message::CandidateOptionPagePrev,
                            next: Message::CandidateOptionPageNext,
                        },
                        field::select(candidates, selected_candidate, |choice| {
                            Message::EntretienCandidatureChanged(choice.id)
                        })
                        .width(iced::Length::Fill)
                        .into(),
                    ),
                ),
                field::labeled(
                    "Contact",
                    relation_page(
                        &app.contact_option_search,
                        Message::ContactOptionSearchChanged,
                        crate::app::state::RELATION_PAGE_SIZE,
                        RelationNavigation {
                            page: app.contact_option_page,
                            total_pages: app.data.contact_options_total_pages,
                            total: app.data.contact_options_total,
                            previous: Message::ContactOptionPagePrev,
                            next: Message::ContactOptionPageNext,
                        },
                        field::select(contacts, selected_contact, |choice| {
                            Message::EntretienContactChanged(choice.value())
                        })
                        .width(iced::Length::Fill)
                        .into(),
                    ),
                ),
            ]
            .spacing(space::LG),
        ),
        field::form_section(
            Icon::Calendar,
            "Organisation",
            "Format, date et modalités pratiques",
            field::form_row([
                field::labeled(
                    "Type",
                    field::select(
                        types,
                        Some(app.entretien_form.type_entretien),
                        Message::EntretienTypeChanged,
                    )
                    .width(iced::Length::Fill),
                ),
                field::datetime_field(
                    "Date et heure *",
                    &app.entretien_form.date_entretien,
                    None,
                    Message::EntretienDateChanged,
                    Message::OpenDatePicker(crate::app::state::DatePickerTarget::Entretien),
                ),
                field::text_field(
                    "Lieu ou lien",
                    &app.entretien_form.lieu,
                    Message::EntretienLieuChanged,
                ),
            ]),
        ),
        field::form_section(
            Icon::Edit,
            "Préparation et suivi",
            "Séparez vos points de préparation du compte rendu final",
            field::form_row([
                field::labeled(
                    "Notes de préparation",
                    field::editor(
                        &app.entretien_form.notes,
                        "Préparation, interlocuteurs, points clés…"
                    )
                    .height(112.0)
                    .on_action(Message::EntretienNotesChanged)
                ),
                field::labeled(
                    "Compte rendu",
                    field::editor(
                        &app.entretien_form.compte_rendu,
                        "Décrivez les échanges, les questions et les prochaines étapes…",
                    )
                    .on_action(Message::EntretienCompteRenduChanged)
                    .height(iced::Length::Fixed(132.0))
                ),
            ]),
        ),
        typo::caption("Date et heure attendues : JJ-MM-AAAA HH:MM."),
    ]
    .spacing(space::LG)
    .into()
}
