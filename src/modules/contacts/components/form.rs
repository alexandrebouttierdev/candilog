//! Formulaire de création/édition d'un contact.

use crate::app::{App, Message};
use crate::modules::candidatures::views::company_choices;
use crate::ui::components::choice::Choice;
use crate::ui::components::field;
use crate::ui::components::icon::Icon;
use crate::ui::components::relation::{relation_page, RelationNavigation};
use crate::ui::theme::metrics::space;
use iced::widget::column;
use iced::Element;

/// Formulaire de création/édition d'un contact.
pub fn form(app: &App) -> Element<'_, Message> {
    let companies = company_choices(app);
    let selected = Choice::find(&companies, app.contact_form.entreprise_id);
    column![
        field::form_section(
            Icon::Profile,
            "Identité",
            "Les informations essentielles du contact",
            field::form_row([
                field::text_field(
                    "Prénom *",
                    &app.contact_form.prenom,
                    Message::ContactPrenomChanged,
                ),
                field::text_field("Nom *", &app.contact_form.nom, Message::ContactNomChanged),
            ])
        ),
        field::form_section(
            Icon::Building,
            "Contexte professionnel",
            "Entreprise suivie et fonction occupée",
            column![
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
                field::text_field(
                    "Poste",
                    &app.contact_form.poste,
                    Message::ContactPosteChanged,
                ),
            ]
            .spacing(space::LG),
        ),
        field::form_section(
            Icon::Mail,
            "Coordonnées",
            "Moyens de contact et présence professionnelle",
            column![
                field::form_row([field::text_field(
                    "E-mail",
                    &app.contact_form.email,
                    Message::ContactEmailChanged
                ),]),
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
            ]
            .spacing(space::LG),
        ),
        field::form_section(
            Icon::Edit,
            "Notes relationnelles",
            "Contexte, historique et prochaine action utile",
            field::editor(
                &app.contact_form.notes,
                "Contexte, historique et informations utiles…"
            )
            .on_action(Message::ContactNotesChanged)
            .height(iced::Length::Fixed(118.0)),
        ),
    ]
    .spacing(space::LG)
    .into()
}
