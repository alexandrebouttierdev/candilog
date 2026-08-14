//! Formulaire de création/édition d'un contact.

use crate::app::{App, Message};
use crate::modules::candidatures::views::company_choices;
use crate::ui::components::choice::Choice;
use crate::ui::components::field;
use crate::ui::components::relation::{relation_page, RelationNavigation};
use crate::ui::theme::metrics::space;
use iced::widget::column;
use iced::Element;

/// Formulaire de création/édition d'un contact.
pub fn form(app: &App) -> Element<'_, Message> {
    let companies = company_choices(app);
    let selected = Choice::find(&companies, app.contact_form.entreprise_id);
    column![
        field::form_row([
            field::text_field(
                "Prénom *",
                &app.contact_form.prenom,
                Message::ContactPrenomChanged,
            ),
            field::text_field("Nom *", &app.contact_form.nom, Message::ContactNomChanged),
        ]),
        field::labeled(
            "Entreprise",
            relation_page(
                &app.company_option_search,
                Message::CompanyOptionSearchChanged,
                crate::app::state::RELATION_PAGE_SIZE,
                RelationNavigation {
                    page: app.company_option_page,
                    total_pages: app.data.company_options_total_pages,
                    total: app.data.company_options_total,
                    previous: Message::CompanyOptionPagePrev,
                    next: Message::CompanyOptionPageNext,
                },
                field::select(companies, selected, |choice| {
                    Message::ContactEntrepriseChanged(choice.value())
                })
                .width(iced::Length::Fill)
                .into(),
            ),
        ),
        field::form_row([
            field::text_field(
                "Poste",
                &app.contact_form.poste,
                Message::ContactPosteChanged
            ),
            field::text_field(
                "E-mail",
                &app.contact_form.email,
                Message::ContactEmailChanged
            ),
        ]),
        field::form_row([
            field::text_field(
                "Téléphone",
                &app.contact_form.telephone,
                Message::ContactTelephoneChanged,
            ),
            field::text_field(
                "LinkedIn",
                &app.contact_form.linkedin,
                Message::ContactLinkedinChanged,
            ),
        ]),
        field::labeled(
            "Notes",
            field::editor(
                &app.contact_form.notes,
                "Contexte, historique et informations utiles…"
            )
            .on_action(Message::ContactNotesChanged)
            .height(iced::Length::Fixed(120.0)),
        ),
    ]
    .spacing(space::LG)
    .into()
}
