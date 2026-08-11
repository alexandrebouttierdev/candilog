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
            (!app.entreprise_form.nom.trim().is_empty()).then_some(Message::SubmitEntreprise),
            Size::Form,
        ),
        Dialog::Contact => form(
            title(app, "Nouveau contact", "Modifier le contact"),
            contact(app),
            (!app.contact_form.prenom.trim().is_empty() && !app.contact_form.nom.trim().is_empty())
                .then_some(Message::SubmitContact),
            Size::Form,
        ),
        Dialog::Candidature => form(
            title(app, "Nouvelle candidature", "Modifier la candidature"),
            candidature(app),
            (!app.candidature_form.poste.trim().is_empty()
                && app.candidature_form.entreprise_id.is_some()
                && !app.candidature_form.date_envoi.trim().is_empty())
            .then_some(Message::SubmitCandidature),
            Size::Form,
        ),
        Dialog::Entretien => form(
            title(app, "Nouvel entretien", "Modifier l'entretien"),
            entretien(app),
            (app.entretien_form.candidature_id.is_some()
                && !app.entretien_form.date_entretien.trim().is_empty())
            .then_some(Message::SubmitEntretien),
            Size::Wide,
        ),
        Dialog::Relance => form(
            title(app, "Nouvelle relance", "Modifier la relance"),
            relance(app),
            (app.relance_form.candidature_id.is_some()
                && !app.relance_form.date_relance.trim().is_empty())
            .then_some(Message::SubmitRelance),
            Size::Form,
        ),
        Dialog::Profil => form(
            "Modifier le profil",
            profile(app),
            Some(Message::SubmitProfile),
            Size::Wide,
        ),
        Dialog::DeleteCandidature(id) => confirm_owned(
            "Supprimer cette candidature",
            consequences_candidature(app, id),
            "Supprimer définitivement",
            Message::ConfirmDelete,
        ),
        Dialog::DeleteEntreprise(id) => confirm_owned(
            "Supprimer cette entreprise",
            consequences_entreprise(app, id),
            "Supprimer définitivement",
            Message::ConfirmDelete,
        ),
        Dialog::DeleteContact(_) => confirm(
            "Supprimer ce contact",
            "Le contact sera supprimé. Les candidatures et entretiens auxquels il est associé \
             sont conservés, et perdent simplement ce rattachement.",
            "Supprimer définitivement",
            Message::ConfirmDelete,
        ),
        Dialog::DeleteEntretien(_) => confirm(
            "Supprimer cet entretien",
            "L'entretien sera supprimé, avec sa date, son lieu, ses notes, son compte rendu et \
             son analyse IA. La candidature associée est conservée.",
            "Supprimer définitivement",
            Message::ConfirmDelete,
        ),
        Dialog::DeleteRelance(_) => confirm(
            "Supprimer cette relance",
            "La relance sera supprimée. La candidature associée est conservée.",
            "Supprimer définitivement",
            Message::ConfirmDelete,
        ),
        Dialog::DeleteCv(_) => confirm(
            "Supprimer cette version de CV",
            "La version enregistrée sera supprimée. Vos candidatures ne sont pas affectées.",
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

/// Modale de formulaire. `submit` vaut `None` tant que la saisie est incomplète.
///
/// Le message de soumission était auparavant câblé **sans condition** : « Enregistrer » était
/// rendu dans son style primaire actif alors que des champs marqués obligatoires par un
/// astérisque étaient vides. La validation n'intervenait qu'après le clic, côté service, et son
/// résultat n'apparaissait que sous forme de toast dans le coin inférieur droit de la fenêtre —
/// à l'opposé du champ fautif, sans qu'aucun champ ne soit signalé.
///
/// Un `on_press` absent grise le bouton : c'est le mécanisme d'état désactivé d'Iced.
fn form<'a>(
    heading: &'a str,
    body: Element<'a, Message>,
    submit: Option<Message>,
    kind: Size,
) -> Element<'a, Message> {
    let mut enregistrer = controls::primary("Enregistrer", Some(Icon::Check));
    if let Some(message) = submit {
        enregistrer = enregistrer.on_press(message);
    }
    overlay::modal(
        heading,
        body,
        overlay::footer([
            controls::ghost("Annuler", None)
                .on_press(Message::CloseDialog)
                .into(),
            enregistrer.into(),
        ]),
        kind,
        Message::CloseDialog,
    )
}

/// Décrit ce que la suppression d'une candidature va réellement emporter.
///
/// Les six confirmations partageaient une formulation unique — « Cette suppression applique les
/// contraintes SQLite et ne peut pas être annulée » — qui expose un détail d'implémentation
/// sans signification pour l'utilisateur et, surtout, ne dit pas ce qui va disparaître : une
/// candidature efface en cascade ses relances et ses entretiens, une entreprise référencée voit
/// au contraire sa suppression refusée. Le texte était identique dans les deux cas.
fn consequences_candidature(app: &App, id: uuid::Uuid) -> String {
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
            parties.push(crate::ui::format::plural(relances, "relance", "relances"));
        }
        if entretiens > 0 {
            parties.push(crate::ui::format::plural(
                entretiens,
                "entretien",
                "entretiens",
            ));
        }
        phrase.push_str(&parties.join(" et "));
    }
    phrase.push_str(". Cette action est définitive.");
    phrase
}

/// Décrit le refus attendu quand l'entreprise porte encore des candidatures (RESTRICT).
fn consequences_entreprise(app: &App, id: uuid::Uuid) -> String {
    let nom = app
        .data
        .entreprises
        .iter()
        .find(|item| item.id == id)
        .map_or_else(|| "Cette entreprise".to_owned(), |item| item.nom.clone());
    let liees = app
        .data
        .candidatures
        .iter()
        .filter(|item| item.entreprise_id == id)
        .count();
    if liees > 0 {
        format!(
            "« {nom} » ne peut pas être supprimée : {} y {} encore rattachée{}. Supprimez-les \
             d'abord, ou conservez l'entreprise.",
            crate::ui::format::plural(liees, "candidature", "candidatures"),
            if liees > 1 { "sont" } else { "est" },
            if liees > 1 { "s" } else { "" },
        )
    } else {
        format!("« {nom} » sera supprimée. Cette action est définitive.")
    }
}

/// Variante de [`confirm`] pour un avertissement calculé.
fn confirm_owned<'a>(
    heading: &'a str,
    warning: String,
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
            controls::danger(action, Some(Icon::Trash))
                .on_press(message)
                .into(),
        ]),
        Size::Form,
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
            field::date_field(
                "Date d'envoi *",
                &app.candidature_form.date_envoi,
                None,
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
        typo::caption("Toutes les dates sont saisies au format JJ-MM-AAAA."),
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
            field::datetime_field(
                "Date et heure *",
                &app.entretien_form.date_entretien,
                None,
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
        field::labeled(
            "Compte rendu",
            field::editor(
                &app.entretien_form.compte_rendu,
                "Décrivez les échanges, les questions et les prochaines étapes…",
            )
            .on_action(Message::EntretienCompteRenduChanged)
            .height(iced::Length::Fixed(132.0)),
        ),
        typo::caption("Date et heure attendues : JJ-MM-AAAA HH:MM."),
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
            field::date_field(
                "Date *",
                &app.relance_form.date_relance,
                None,
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
