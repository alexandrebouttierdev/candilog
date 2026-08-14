//! Sous-domaine d'opérations applicatives.

use super::super::*;

pub(super) fn handles(message: &Message) -> bool {
    matches!(
        message,
        Message::CandidateFilterStatus(..)
            | Message::CandidateFilterContract(..)
            | Message::CandidateFilterCompany(..)
            | Message::CandidateFilterCity(..)
            | Message::CandidateFilterPosition(..)
            | Message::CandidateFilterDateFrom(..)
            | Message::CandidateFilterDateTo(..)
            | Message::ResetCandidateFilters
            | Message::ConfirmDelete
            | Message::EditEntreprise(..)
            | Message::EditContact(..)
            | Message::EditCandidature(..)
            | Message::EditEntretien(..)
            | Message::EditRelance(..)
    )
}

pub(super) fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::CandidateFilterStatus(value) => {
            app.candidate_filters.status = value;
            app.candidate_page = 1;
            return recharger(app);
        }
        Message::CandidateFilterContract(value) => {
            app.candidate_filters.contract = value;
            app.candidate_page = 1;
            return recharger(app);
        }
        Message::CandidateFilterCompany(value) => {
            app.candidate_filters.company_id = value;
            app.candidate_page = 1;
            return recharger(app);
        }
        Message::CandidateFilterCity(value) => {
            app.candidate_filters.city = value;
            app.candidate_page = 1;
            return recharger(app);
        }
        Message::CandidateFilterPosition(value) => {
            app.candidate_filters.position = value;
            app.candidate_page = 1;
            return recharger(app);
        }
        Message::CandidateFilterDateFrom(value) => {
            app.candidate_filters.date_from = value;
            app.candidate_page = 1;
            return recharger(app);
        }
        Message::CandidateFilterDateTo(value) => {
            app.candidate_filters.date_to = value;
            app.candidate_page = 1;
            return recharger(app);
        }
        Message::ResetCandidateFilters => {
            app.candidate_filters = crate::app::state::CandidateFilters::default();
            app.candidate_page = 1;
            return recharger(app);
        }
        Message::ConfirmDelete => {
            let Some(dialog) = app.dialog else {
                return Task::none();
            };
            return ecrire(app, "Élément supprimé.", move |backend| {
                match dialog {
                    Dialog::DeleteCandidature(id) => backend.candidatures.supprimer(id),
                    Dialog::DeleteEntreprise(id) => backend.entreprises.supprimer(id),
                    Dialog::DeleteContact(id) => backend.contacts.supprimer(id),
                    Dialog::DeleteEntretien(id) => backend.entretiens.supprimer(id),
                    Dialog::DeleteRelance(id) => backend.relances.supprimer(id),
                    Dialog::DeleteCv(id) => backend.cv.delete(id),
                    _ => Err(crate::shared::error::AppError::Validation(
                        "Aucune suppression à confirmer.".into(),
                    )),
                }
                .map_err(|error| error.to_string())
            });
        }
        Message::EditEntreprise(id) => {
            if let Some(item) = app.data.entreprises.iter().find(|item| item.id == id) {
                app.entreprise_form = EntrepriseForm {
                    nom: item.nom.clone(),
                    secteur: item.secteur.clone().unwrap_or_default(),
                    type_: item.type_.clone().unwrap_or_default(),
                    site_web: item.site_web.clone().unwrap_or_default(),
                    ville: item.ville.clone().unwrap_or_default(),
                    adresse: item.adresse.clone().unwrap_or_default(),
                    notes: iced::widget::text_editor::Content::with_text(
                        item.notes.as_deref().unwrap_or_default(),
                    ),
                };
                app.editing_id = Some(id);
                app.dialog = Some(Dialog::Entreprise);
            }
        }
        Message::EditContact(id) => {
            if let Some(item) = app.data.contacts.iter().find(|item| item.id == id) {
                app.contact_form = ContactForm {
                    entreprise_id: item.entreprise_id,
                    prenom: item.prenom.clone(),
                    nom: item.nom.clone(),
                    poste: item.poste.clone().unwrap_or_default(),
                    email: item.email.clone().unwrap_or_default(),
                    telephone: item.telephone.clone().unwrap_or_default(),
                    linkedin: item.linkedin.clone().unwrap_or_default(),
                    notes: iced::widget::text_editor::Content::with_text(
                        item.notes.as_deref().unwrap_or_default(),
                    ),
                };
                app.editing_id = Some(id);
                app.dialog = Some(Dialog::Contact);
            }
        }
        Message::EditCandidature(id) => {
            if let Some(item) = app.data.candidatures.iter().find(|item| item.id == id) {
                app.candidature_form = CandidatureForm {
                    entreprise_id: Some(item.entreprise_id),
                    poste: item.poste.clone(),
                    type_contrat: item.type_contrat,
                    statut: item.statut,
                    date_envoi: ui_format::date_for_input(&item.date_envoi),
                    lien_offre: item.lien_offre.clone().unwrap_or_default(),
                    notes: item.notes.clone().unwrap_or_default(),
                };
                app.editing_id = Some(id);
                app.dialog = Some(Dialog::Candidature);
            }
        }
        Message::EditEntretien(id) => {
            if let Some(item) = app.data.entretiens.iter().find(|item| item.id == id) {
                app.entretien_form = EntretienForm {
                    candidature_id: Some(item.candidature_id),
                    contact_id: item.contact_id,
                    date_entretien: ui_format::datetime_for_input(&item.date_entretien),
                    type_entretien: item.type_entretien,
                    lieu: item.lieu.clone().unwrap_or_default(),
                    notes: iced::widget::text_editor::Content::with_text(
                        item.notes.as_deref().unwrap_or_default(),
                    ),
                    compte_rendu: iced::widget::text_editor::Content::with_text(
                        item.compte_rendu.as_deref().unwrap_or_default(),
                    ),
                };
                app.editing_id = Some(id);
                app.dialog = Some(Dialog::Entretien);
            }
        }
        Message::EditRelance(id) => {
            if let Some(item) = app.data.relances.iter().find(|item| item.id == id) {
                app.relance_form = RelanceForm {
                    candidature_id: Some(item.candidature_id),
                    date_relance: ui_format::date_for_input(&item.date_relance),
                    type_relance: item.type_relance.clone(),
                    notes: item.notes.clone().unwrap_or_default(),
                };
                app.editing_id = Some(id);
                app.dialog = Some(Dialog::Relance);
            }
        }
        _ => unreachable!("message routé vers un sous-domaine incorrect"),
    }
    Task::none()
}
