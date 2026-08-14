//! Formulaire de relance (création et édition).

use crate::app::{App, Message};
use crate::modules::candidatures::views::candidate_choices;
use crate::ui::components::choice::Choice;
use crate::ui::components::field;
use crate::ui::components::relation::{relation_page, RelationNavigation};
use crate::ui::theme::metrics::space;
use iced::widget::column;
use iced::Element;

/// Canaux de relance proposés.
pub fn channels() -> Vec<String> {
    vec![
        "Email".to_owned(),
        "Téléphone".to_owned(),
        "LinkedIn".to_owned(),
        "Autre".to_owned(),
    ]
}

/// Formulaire de création/édition d'une relance.
pub fn form(app: &App) -> Element<'_, Message> {
    let candidates = candidate_choices(app);
    let selected = Choice::find(&candidates, app.relance_form.candidature_id);
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
                field::select(candidates, selected, |choice| {
                    Message::RelanceCandidatureChanged(choice.id)
                })
                .width(iced::Length::Fill)
                .into(),
            ),
        ),
        field::form_row([
            field::date_field(
                "Date *",
                &app.relance_form.date_relance,
                None,
                Message::RelanceDateChanged,
                Message::OpenDatePicker(crate::app::state::DatePickerTarget::Relance),
            ),
            field::labeled(
                "Canal",
                field::select(
                    channels(),
                    Some(app.relance_form.type_relance.clone()),
                    Message::RelanceTypeChanged,
                )
                .width(iced::Length::Fill),
            ),
        ]),
        field::text_field(
            "Notes",
            &app.relance_form.notes,
            Message::RelanceNotesChanged
        ),
    ]
    .spacing(space::LG)
    .into()
}
