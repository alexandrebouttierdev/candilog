//! Dialogues métier : formulaires, confirmations et inspecteur latéral.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::ui::components::button as controls;
use crate::ui::components::icon::Icon;
use crate::ui::components::overlay::{self, Size};
use crate::ui::components::state;
use iced::Element;

/// Rend la couche superposée correspondant au dialogue ouvert.
pub fn layer(app: &App, dialog: Dialog) -> Element<'_, Message> {
    if let Dialog::CandidatureDetail(id) = dialog {
        return super::inspector_layer(app, id);
    }
    if dialog == Dialog::ProfileImport {
        return overlay::wide_drawer(
            crate::modules::profil::views::import_review_drawer(app),
            Message::Noop,
        );
    }

    match dialog {
        Dialog::Entreprise => form(
            app,
            title(app, "Nouvelle entreprise", "Modifier l'entreprise"),
            crate::modules::entreprises::components::form::form(app),
            (!app.entreprise_form.nom.trim().is_empty()).then_some(Message::SubmitEntreprise),
            Size::Form,
        ),
        Dialog::Contact => form(
            app,
            title(app, "Nouveau contact", "Modifier le contact"),
            crate::modules::contacts::components::form::form(app),
            (!app.contact_form.prenom.trim().is_empty() && !app.contact_form.nom.trim().is_empty())
                .then_some(Message::SubmitContact),
            Size::Form,
        ),
        Dialog::Candidature => form(
            app,
            title(app, "Nouvelle candidature", "Modifier la candidature"),
            crate::modules::candidatures::components::form::form(app),
            (!app.candidature_form.poste.trim().is_empty()
                && app.candidature_form.entreprise_id.is_some()
                && !app.candidature_form.date_envoi.trim().is_empty())
            .then_some(Message::SubmitCandidature),
            Size::Form,
        ),
        Dialog::Entretien => form(
            app,
            title(app, "Nouvel entretien", "Modifier l'entretien"),
            crate::modules::entretiens::components::form::form(app),
            (app.entretien_form.candidature_id.is_some()
                && !app.entretien_form.date_entretien.trim().is_empty())
            .then_some(Message::SubmitEntretien),
            Size::Wide,
        ),
        Dialog::Relance => form(
            app,
            title(app, "Nouvelle relance", "Modifier la relance"),
            crate::modules::relances::components::form::form(app),
            (app.relance_form.candidature_id.is_some()
                && !app.relance_form.date_relance.trim().is_empty())
            .then_some(Message::SubmitRelance),
            Size::Form,
        ),
        Dialog::Profil(section) => form(
            app,
            crate::modules::profil::components::form::dialog_title(section),
            crate::modules::profil::components::form::form(app, section),
            Some(Message::SubmitProfile),
            Size::Wide,
        ),
        Dialog::ProfileImport => unreachable!("traité comme un drawer avant le match"),
        Dialog::DeleteCandidature(id) => confirm_owned(
            app,
            "Supprimer cette candidature",
            crate::modules::candidatures::components::form::consequences_suppression(app, id),
            "Supprimer définitivement",
            Message::ConfirmDelete,
        ),
        Dialog::DeleteEntreprise(id) => confirm_owned(
            app,
            "Supprimer cette entreprise",
            crate::modules::entreprises::components::form::consequences_suppression(app, id),
            "Supprimer définitivement",
            Message::ConfirmDelete,
        ),
        Dialog::DeleteContact(_) => confirm(
            app,
            "Supprimer ce contact",
            "Le contact sera supprimé. Les candidatures et entretiens auxquels il est associé \
             sont conservés, et perdent simplement ce rattachement.",
            "Supprimer définitivement",
            Message::ConfirmDelete,
        ),
        Dialog::DeleteEntretien(_) => confirm(
            app,
            "Supprimer cet entretien",
            "L'entretien sera supprimé, avec sa date, son lieu, ses notes, son compte rendu et \
             son analyse IA. La candidature associée est conservée.",
            "Supprimer définitivement",
            Message::ConfirmDelete,
        ),
        Dialog::DeleteRelance(_) => confirm(
            app,
            "Supprimer cette relance",
            "La relance sera supprimée. La candidature associée est conservée.",
            "Supprimer définitivement",
            Message::ConfirmDelete,
        ),
        Dialog::DeleteCv(_) => confirm(
            app,
            "Supprimer cette version de CV",
            "La version enregistrée sera supprimée. Vos candidatures ne sont pas affectées.",
            "Supprimer définitivement",
            Message::ConfirmDelete,
        ),
        Dialog::ImportBackup => confirm(
            app,
            "Restaurer le backup",
            "Toutes les données actuelles seront remplacées par le backup validé.",
            "Restaurer maintenant",
            Message::ConfirmBackupImport,
        ),
        Dialog::ResetDatabase => confirm(
            app,
            "Réinitialiser Candilog",
            "Toutes les données locales seront définitivement supprimées.",
            "Tout réinitialiser",
            Message::ConfirmDatabaseReset,
        ),
        Dialog::ResetAiCache => confirm(
            app,
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
    app: &App,
    heading: &'a str,
    body: Element<'a, Message>,
    submit: Option<Message>,
    kind: Size,
) -> Element<'a, Message> {
    let mut enregistrer = controls::primary("Enregistrer", Some(Icon::Check));
    if let Some(message) = submit.filter(|_| !app.write_in_progress) {
        enregistrer = enregistrer.on_press(message);
    }
    let mut annuler = controls::ghost("Annuler", None);
    if !app.write_in_progress {
        annuler = annuler.on_press(Message::CloseDialog);
    }
    overlay::modal(
        heading,
        body,
        overlay::footer([annuler.into(), enregistrer.into()]),
        kind,
        if app.write_in_progress {
            Message::Noop
        } else {
            Message::CloseDialog
        },
        Message::Noop,
    )
}

/// Variante de [`confirm`] pour un avertissement calculé.
fn confirm_owned<'a>(
    app: &App,
    heading: &'a str,
    warning: String,
    action: &'a str,
    message: Message,
) -> Element<'a, Message> {
    let mut annuler = controls::ghost("Annuler", None);
    if !app.write_in_progress {
        annuler = annuler.on_press(Message::CloseDialog);
    }
    overlay::modal(
        heading,
        state::error(warning),
        overlay::footer([annuler.into(), {
            let mut action_button = controls::danger(action, Some(Icon::Trash));
            if !app.write_in_progress {
                action_button = action_button.on_press(message);
            }
            action_button.into()
        }]),
        Size::Form,
        if app.write_in_progress {
            Message::Noop
        } else {
            Message::CloseDialog
        },
        Message::Noop,
    )
}

fn confirm<'a>(
    app: &App,
    heading: &'a str,
    warning: &'a str,
    action: &'a str,
    message: Message,
) -> Element<'a, Message> {
    let mut annuler = controls::ghost("Annuler", None);
    if !app.write_in_progress {
        annuler = annuler.on_press(Message::CloseDialog);
    }
    overlay::modal(
        heading,
        state::error(warning),
        overlay::footer([annuler.into(), {
            let mut action_button = controls::danger_filled(action);
            if !app.write_in_progress {
                action_button = action_button.on_press(message);
            }
            action_button.into()
        }]),
        Size::Confirm,
        if app.write_in_progress {
            Message::Noop
        } else {
            Message::CloseDialog
        },
        Message::Noop,
    )
}
