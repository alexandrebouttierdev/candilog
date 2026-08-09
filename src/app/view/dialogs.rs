//! Dialogues métier : formulaires, confirmations et inspecteur latéral.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::candidatures::components::PIPELINE;
use crate::modules::candidatures::model::TypeContrat;
use crate::modules::candidatures::views::{candidate_choices, company_choices};
use crate::modules::entretiens::model::TypeEntretien;
use crate::ui::components::button as controls;
use crate::ui::components::choice::Choice;
use crate::ui::components::icon::Icon;
use crate::ui::components::overlay::{self, Size};
use crate::ui::components::{field, state, typo};
use crate::ui::theme::metrics::space;
use crate::ui::theme::Tone;
use iced::widget::column;
use iced::Element;

/// Canaux de relance proposés.
fn channels() -> Vec<String> {
    vec![
        "Email".to_owned(),
        "Téléphone".to_owned(),
        "LinkedIn".to_owned(),
        "Autre".to_owned(),
    ]
}

/// Rend la couche superposée correspondant au dialogue ouvert.
pub fn layer(app: &App, dialog: Dialog) -> Element<'_, Message> {
    if let Dialog::CandidatureDetail(id) = dialog {
        return super::inspector_layer(app, id);
    }

    match dialog {
        Dialog::Entreprise => form(
            title(app, "Nouvelle entreprise", "Modifier l'entreprise"),
            entreprise(app),
            Message::SubmitEntreprise,
            Size::Form,
        ),
        Dialog::Contact => form(
            title(app, "Nouveau contact", "Modifier le contact"),
            contact(app),
            Message::SubmitContact,
            Size::Form,
        ),
        Dialog::Candidature => form(
            title(app, "Nouvelle candidature", "Modifier la candidature"),
            candidature(app),
            Message::SubmitCandidature,
            Size::Form,
        ),
        Dialog::Entretien => form(
            title(app, "Nouvel entretien", "Modifier l'entretien"),
            entretien(app),
            Message::SubmitEntretien,
            Size::Wide,
        ),
        Dialog::Relance => form(
            title(app, "Nouvelle relance", "Modifier la relance"),
            relance(app),
            Message::SubmitRelance,
            Size::Form,
        ),
        Dialog::Profil => form(
            "Modifier le profil",
            profile(app),
            Message::SubmitProfile,
            Size::Wide,
        ),
        Dialog::DeleteCandidature(_)
        | Dialog::DeleteEntreprise(_)
        | Dialog::DeleteContact(_)
        | Dialog::DeleteEntretien(_)
        | Dialog::DeleteRelance(_)
        | Dialog::DeleteCv(_) => confirm(
            "Confirmer la suppression",
            "Cette suppression applique les contraintes SQLite et ne peut pas être annulée.",
            "Supprimer définitivement",
            Message::ConfirmDelete,
        ),
        Dialog::ImportBackup => confirm(
            "Restaurer le backup",
            "Toutes les données actuelles seront remplacées par le backup validé.",
            "Restaurer maintenant",
            Message::ConfirmBackupImport,
        ),
        Dialog::ResetDatabase => confirm(
            "Réinitialiser Candilog",
            "Toutes les données locales seront définitivement supprimées.",
            "Tout réinitialiser",
            Message::ConfirmDatabaseReset,
        ),
        Dialog::ResetAiCache => confirm(
            "Vider le cache IA",
            "Les résultats IA mémorisés seront supprimés. Les données métier sont conservées.",
            "Vider le cache",
            Message::ConfirmAiCacheReset,
        ),
        Dialog::CandidatureDetail(id) => super::inspector_layer(app, id),
    }
}

fn title<'a>(app: &App, creation: &'a str, edition: &'a str) -> &'a str {
    if app.editing_id.is_some() {
        edition
    } else {
        creation
    }
}

fn form<'a>(
    heading: &'a str,
    body: Element<'a, Message>,
    submit: Message,
    kind: Size,
) -> Element<'a, Message> {
    overlay::modal(
        heading,
        body,
        overlay::footer([
            controls::ghost("Annuler", None)
                .on_press(Message::CloseDialog)
                .into(),
            controls::primary("Enregistrer", Some(Icon::Check))
                .on_press(submit)
                .into(),
        ]),
        kind,
        Message::CloseDialog,
    )
}

fn confirm<'a>(
    heading: &'a str,
    warning: &'a str,
    action: &'a str,
    message: Message,
) -> Element<'a, Message> {
    overlay::modal(
        heading,
        state::error(warning),
        overlay::footer([
            controls::ghost("Annuler", None)
                .on_press(Message::CloseDialog)
                .into(),
            controls::danger_filled(action).on_press(message).into(),
        ]),
        Size::Confirm,
        Message::CloseDialog,
    )
}

fn entreprise(app: &App) -> Element<'_, Message> {
    column![
        field::text_field(
            "Nom *",
            &app.entreprise_form.nom,
            Message::EntrepriseNomChanged
        ),
        field::form_row([
            field::text_field(
                "Secteur",
                &app.entreprise_form.secteur,
                Message::EntrepriseSecteurChanged,
            ),
            field::text_field(
                "Type",
                &app.entreprise_form.type_,
                Message::EntrepriseTypeChanged,
            ),
        ]),
        field::form_row([
            field::text_field(
                "Site web",
                &app.entreprise_form.site_web,
                Message::EntrepriseSiteChanged,
            ),
            field::text_field(
                "Ville",
                &app.entreprise_form.ville,
                Message::EntrepriseVilleChanged,
            ),
        ]),
        field::text_field(
            "Adresse",
            &app.entreprise_form.adresse,
            Message::EntrepriseAdresseChanged,
        ),
        field::text_field(
            "Notes",
            &app.entreprise_form.notes,
            Message::EntrepriseNotesChanged,
        ),
    ]
    .spacing(space::LG)
    .into()
}

fn contact(app: &App) -> Element<'_, Message> {
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
            field::select(companies, selected, |choice| {
                Message::ContactEntrepriseChanged(choice.value())
            })
            .width(iced::Length::Fill),
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
        field::text_field(
            "Notes",
            &app.contact_form.notes,
            Message::ContactNotesChanged
        ),
    ]
    .spacing(space::LG)
    .into()
}

fn candidature(app: &App) -> Element<'_, Message> {
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
            field::select(companies, selected, |choice| {
                Message::CandidatureEntrepriseChanged(choice.id)
            })
            .width(iced::Length::Fill),
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
            field::text_field(
                "Date d'envoi *",
                &app.candidature_form.date_envoi,
                Message::CandidatureDateChanged,
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
        typo::caption("Le format attendu pour la date est AAAA-MM-JJ."),
    ]
    .spacing(space::LG)
    .into()
}

fn entretien(app: &App) -> Element<'_, Message> {
    let candidates = candidate_choices(app);
    let selected_candidate = Choice::find(&candidates, app.entretien_form.candidature_id);
    let contacts: Vec<Choice> = app
        .data
        .contacts
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
        field::labeled(
            "Candidature *",
            field::select(candidates, selected_candidate, |choice| {
                Message::EntretienCandidatureChanged(choice.id)
            })
            .width(iced::Length::Fill),
        ),
        field::form_row([
            field::labeled(
                "Contact",
                field::select(contacts, selected_contact, |choice| {
                    Message::EntretienContactChanged(choice.value())
                })
                .width(iced::Length::Fill),
            ),
            field::labeled(
                "Type",
                field::select(
                    types,
                    Some(app.entretien_form.type_entretien),
                    Message::EntretienTypeChanged,
                )
                .width(iced::Length::Fill),
            ),
        ]),
        field::form_row([
            field::text_field(
                "Date et heure *",
                &app.entretien_form.date_entretien,
                Message::EntretienDateChanged,
            ),
            field::text_field(
                "Lieu ou lien",
                &app.entretien_form.lieu,
                Message::EntretienLieuChanged,
            ),
        ]),
        field::text_field(
            "Notes",
            &app.entretien_form.notes,
            Message::EntretienNotesChanged
        ),
        field::text_field(
            "Compte rendu",
            &app.entretien_form.compte_rendu,
            Message::EntretienCompteRenduChanged,
        ),
        typo::caption("Le format attendu est AAAA-MM-JJTHH:MM."),
    ]
    .spacing(space::LG)
    .into()
}

fn relance(app: &App) -> Element<'_, Message> {
    let candidates = candidate_choices(app);
    let selected = Choice::find(&candidates, app.relance_form.candidature_id);
    column![
        field::labeled(
            "Candidature *",
            field::select(candidates, selected, |choice| {
                Message::RelanceCandidatureChanged(choice.id)
            })
            .width(iced::Length::Fill),
        ),
        field::form_row([
            field::text_field(
                "Date *",
                &app.relance_form.date_relance,
                Message::RelanceDateChanged,
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

fn profile(app: &App) -> Element<'_, Message> {
    let personal = &app.profile_personal_form;
    column![
        field::form_row([
            field::text_field(
                "Prénom",
                &personal.first_name,
                Message::ProfileFirstNameChanged,
            ),
            field::text_field("Nom", &personal.last_name, Message::ProfileLastNameChanged),
        ]),
        field::form_row([
            field::text_field("E-mail", &personal.email, Message::ProfileEmailChanged),
            field::text_field(
                "Téléphone",
                personal.phone.as_deref().unwrap_or_default(),
                Message::ProfilePhoneChanged,
            ),
        ]),
        field::form_row([
            field::text_field(
                "Ville",
                personal.city.as_deref().unwrap_or_default(),
                Message::ProfileCityChanged,
            ),
            field::text_field(
                "Titre professionnel",
                personal.headline.as_deref().unwrap_or_default(),
                Message::ProfileHeadlineChanged,
            ),
        ]),
        field::text_field(
            "Résumé",
            personal.summary.as_deref().unwrap_or_default(),
            Message::ProfileSummaryChanged,
        ),
        field::text_field(
            "Compétences",
            &app.profile_skills_form,
            Message::ProfileSkillsChanged,
        ),
        state::hint("Séparez les compétences par des virgules."),
        typo::meta_toned(
            "Ces informations alimentent le générateur de CV et le score ATS.",
            Tone::Neutral,
        ),
    ]
    .spacing(space::LG)
    .into()
}
