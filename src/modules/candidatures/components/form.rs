//! Formulaire de création/édition d'une candidature et avertissement de
//! suppression calculé depuis les dépendances (relances et entretiens).

use super::PIPELINE;
use crate::app::{App, Message};
use crate::modules::candidatures::model::TypeContrat;
use crate::modules::candidatures::views::company_choices;
use crate::ui::components::choice::Choice;
use crate::ui::components::field;
use crate::ui::components::relation::{relation_page, RelationNavigation};
use crate::ui::components::typo;
use crate::ui::format;
use crate::ui::theme::metrics::space;
use iced::widget::column;
use iced::Element;

/// Formulaire de création/édition d'une candidature.
pub fn form(app: &App) -> Element<'_, Message> {
    let companies: Vec<Choice> = company_choices(app)
        .into_iter()
        .filter(|choice| !choice.id.is_nil())
        .collect();
    let selected = Choice::find(&companies, app.candidature_form.entreprise_id);
    let contracts = vec![
        TypeContrat::Cdi,
        TypeContrat::Cdd,
        TypeContrat::Freelance,
        TypeContrat::Stage,
        TypeContrat::Alternance,
        TypeContrat::Interim,
        TypeContrat::Autre,
    ];

    column![
        field::text_field(
            "Poste *",
            &app.candidature_form.poste,
            Message::CandidaturePosteChanged,
        ),
        field::labeled(
            "Entreprise *",
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
                    Message::CandidatureEntrepriseChanged(choice.id)
                })
                .width(iced::Length::Fill)
                .into(),
            ),
        ),
        field::form_row([
            field::labeled(
                "Contrat",
                field::select(
                    contracts,
                    Some(app.candidature_form.type_contrat),
                    Message::CandidatureContratChanged,
                )
                .width(iced::Length::Fill),
            ),
            field::labeled(
                "Statut",
                field::select(
                    PIPELINE.to_vec(),
                    Some(app.candidature_form.statut),
                    Message::CandidatureStatutChanged,
                )
                .width(iced::Length::Fill),
            ),
        ]),
        field::form_row([
            field::date_field(
                "Date d'envoi *",
                &app.candidature_form.date_envoi,
                None,
                Message::CandidatureDateChanged,
                Message::OpenDatePicker(crate::app::state::DatePickerTarget::Candidature),
            ),
            field::text_field(
                "Lien de l'offre",
                &app.candidature_form.lien_offre,
                Message::CandidatureLienChanged,
            ),
        ]),
        field::text_field(
            "Notes",
            &app.candidature_form.notes,
            Message::CandidatureNotesChanged,
        ),
        typo::caption("Toutes les dates sont saisies au format JJ-MM-AAAA."),
    ]
    .spacing(space::LG)
    .into()
}

/// Décrit ce que la suppression d'une candidature va réellement emporter
/// (relances et entretiens supprimés en cascade).
pub fn consequences_suppression(app: &App, id: uuid::Uuid) -> String {
    let poste = app
        .data
        .candidatures
        .iter()
        .find(|item| item.id == id)
        .map_or_else(|| "Cette candidature".to_owned(), |item| item.poste.clone());
    let relances = app
        .data
        .relances
        .iter()
        .filter(|item| item.candidature_id == id)
        .count();
    let entretiens = app
        .data
        .entretiens
        .iter()
        .filter(|item| item.candidature_id == id)
        .count();
    let mut phrase = format!("« {poste} » sera supprimée");
    if relances > 0 || entretiens > 0 {
        phrase.push_str(", ainsi que ");
        let mut parties = Vec::new();
        if relances > 0 {
            parties.push(format::plural(relances, "relance", "relances"));
        }
        if entretiens > 0 {
            parties.push(format::plural(entretiens, "entretien", "entretiens"));
        }
        phrase.push_str(&parties.join(" et "));
    }
    phrase.push_str(". Cette action est définitive.");
    phrase
}
