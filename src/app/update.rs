//! Traitement des messages Iced.

use super::capture::save_review_screenshot;
use super::commandes::{ecrire, finish_submit, notifier_le_bureau, recharger, sonner_fin_analyse};
use super::export::export_candidatures;
use super::message::{LetterStreamEvent, UpdateDownloadEvent};
use super::profile_edit::{
    add_profile_item, all_import_item_keys, apply_recommendation, filter_imported_profile,
    merge_imported_profile, remove_profile_item, update_profile_item,
};
use super::state::{
    CandidatureForm, ContactForm, DatePickerState, DatePickerTarget, Dialog, EntrepriseForm,
    EntretienForm, NotificationKind, RelanceForm,
};
use super::{App, Message};
use crate::modules::candidatures::model::NouvelleCandidature;
use crate::modules::contacts::model::NouveauContact;
use crate::modules::entreprises::model::NouvelleEntreprise;
use crate::modules::entretiens::model::NouvelEntretien;
use crate::modules::ia::cache::CacheIaRepository;
use crate::modules::relances::model::NouvelleRelance;
use crate::ui::format as ui_format;
use chrono::{Datelike, Local};
use iced::futures::SinkExt;
use iced::Task;
use semver::Version;

mod ai;
mod forms;
mod interactions;
mod operations;
mod settings;
mod ui_state;

/// Met à jour l'état applicatif.
pub fn update(app: &mut App, message: Message) -> Task<Message> {
    if forms::handles(&message) {
        return forms::update(app, message);
    }
    if ai::handles(&message) {
        return ai::update(app, message);
    }
    if settings::handles(&message) {
        return settings::update(app, message);
    }
    if interactions::handles(&message) {
        return interactions::update(app, message);
    }
    if operations::handles(&message) {
        return operations::update(app, message);
    }
    if ui_state::handles(&message) {
        return ui_state::update(app, message);
    }
    match message {
        Message::Noop => {}
        Message::WriteFinished(result, succes) => {
            if result.is_ok() && app.dialog == Some(Dialog::ImportBackup) {
                app.pending_backup_import = None;
            }
            finish_submit(app, result, succes);
            return recharger(app);
        }
        Message::DataLoaded(data, echecs, sequence) => {
            if sequence == app.data_request_sequence {
                app.appliquer_instantane(*data, &echecs);
            }
        }
        Message::CaptureForReview => {
            return iced::window::get_latest().map(Message::CaptureWindow);
        }
        Message::CaptureWindow(Some(id)) => {
            return iced::window::screenshot(id).map(Message::CapturedForReview);
        }
        Message::CaptureWindow(None) => {
            app.notify_failure("Capture visuelle : fenêtre introuvable.");
        }
        Message::CapturedForReview(screenshot) => {
            if let Err(error) = save_review_screenshot(&screenshot) {
                app.notify_failure(error);
            } else {
                return iced::exit();
            }
        }
        Message::MaximizeWindow => {
            return iced::window::get_latest().map(|id| {
                id.map_or(Message::ClearNotification, |window_id| {
                    // Deuxième message : la fenêtre existe, on la maximise.
                    super::Message::MaximizeWindowId(window_id)
                })
            });
        }
        Message::MaximizeWindowId(id) => {
            return iced::window::maximize(id, true);
        }
        Message::WindowResized(size) => {
            app.window_size = size;
        }
        Message::Navigate(route) => {
            let refresh_models = route == crate::navigation::Route::Parametres;
            if route == crate::navigation::Route::Parametres {
                // Le brouillon repart de l'état persisté : quitter l'écran sans enregistrer
                // ne doit pas laisser de modification en mémoire.
                app.settings_form =
                    crate::app::state::SettingsForm::from_settings(&app.data.settings);
            }
            app.route = route;
            app.search.clear();
            app.candidate_page = 1;
            app.company_page = 1;
            app.contact_page = 1;
            let reload = recharger(app);
            return if refresh_models {
                Task::batch([reload, Task::done(Message::RefreshLlmModels)])
            } else {
                reload
            };
        }
        Message::Reload => return recharger(app),
        Message::SearchChanged(value) => {
            app.search = value;
            match app.route {
                crate::navigation::Route::Candidatures => app.candidate_page = 1,
                crate::navigation::Route::Entreprises => app.company_page = 1,
                crate::navigation::Route::Reseau => app.contact_page = 1,
                _ => {}
            }
            return recharger(app);
        }
        Message::CandidatePagePrev => {
            app.candidate_page = app.candidate_page.saturating_sub(1).max(1);
            return recharger(app);
        }
        Message::CandidatePageNext => {
            app.candidate_page = app
                .candidate_page
                .saturating_add(1)
                .min(app.candidate_total_pages());
            return recharger(app);
        }
        Message::CompanyPagePrev => {
            app.company_page = app.company_page.saturating_sub(1).max(1);
            return recharger(app);
        }
        Message::CompanyPageNext => {
            app.company_page = app
                .company_page
                .saturating_add(1)
                .min(app.company_total_pages());
            return recharger(app);
        }
        Message::ContactPagePrev => {
            app.contact_page = app.contact_page.saturating_sub(1).max(1);
            return recharger(app);
        }
        Message::ContactPageNext => {
            app.contact_page = app
                .contact_page
                .saturating_add(1)
                .min(app.contact_total_pages());
            return recharger(app);
        }
        Message::CompanyOptionSearchChanged(value) => {
            app.company_option_search = value;
            app.company_option_page = 1;
            return recharger(app);
        }
        Message::CandidateOptionSearchChanged(value) => {
            app.candidate_option_search = value;
            app.candidate_option_page = 1;
            return recharger(app);
        }
        Message::ContactOptionSearchChanged(value) => {
            app.contact_option_search = value;
            app.contact_option_page = 1;
            return recharger(app);
        }
        Message::CompanyOptionPagePrev => {
            app.company_option_page = app.company_option_page.saturating_sub(1).max(1);
            return recharger(app);
        }
        Message::CompanyOptionPageNext => {
            app.company_option_page = app
                .company_option_page
                .saturating_add(1)
                .min(app.data.company_options_total_pages.max(1));
            return recharger(app);
        }
        Message::CandidateOptionPagePrev => {
            app.candidate_option_page = app.candidate_option_page.saturating_sub(1).max(1);
            return recharger(app);
        }
        Message::CandidateOptionPageNext => {
            app.candidate_option_page = app
                .candidate_option_page
                .saturating_add(1)
                .min(app.data.candidate_options_total_pages.max(1));
            return recharger(app);
        }
        Message::ContactOptionPagePrev => {
            app.contact_option_page = app.contact_option_page.saturating_sub(1).max(1);
            return recharger(app);
        }
        Message::ContactOptionPageNext => {
            app.contact_option_page = app
                .contact_option_page
                .saturating_add(1)
                .min(app.data.contact_options_total_pages.max(1));
            return recharger(app);
        }
        Message::CandidateViewChanged(mode) => app.candidate_view = mode,
        Message::PreviousMonth => {
            match app.calendar_view {
                super::message::CalendarView::Month => {
                    if app.calendar_month == 1 {
                        app.calendar_month = 12;
                        app.calendar_year -= 1;
                    } else {
                        app.calendar_month -= 1;
                    }
                }
                super::message::CalendarView::Week => {
                    app.calendar_date -= chrono::Duration::days(7);
                    app.calendar_year = app.calendar_date.year();
                    app.calendar_month = app.calendar_date.month();
                }
                super::message::CalendarView::Day => {
                    app.calendar_date -= chrono::Duration::days(1);
                    app.calendar_year = app.calendar_date.year();
                    app.calendar_month = app.calendar_date.month();
                }
            }
            return recharger(app);
        }
        Message::NextMonth => {
            match app.calendar_view {
                super::message::CalendarView::Month => {
                    if app.calendar_month == 12 {
                        app.calendar_month = 1;
                        app.calendar_year += 1;
                    } else {
                        app.calendar_month += 1;
                    }
                }
                super::message::CalendarView::Week => {
                    app.calendar_date += chrono::Duration::days(7);
                    app.calendar_year = app.calendar_date.year();
                    app.calendar_month = app.calendar_date.month();
                }
                super::message::CalendarView::Day => {
                    app.calendar_date += chrono::Duration::days(1);
                    app.calendar_year = app.calendar_date.year();
                    app.calendar_month = app.calendar_date.month();
                }
            }
            return recharger(app);
        }
        Message::CurrentMonth => {
            let now = Local::now();
            app.calendar_year = now.year();
            app.calendar_month = now.month();
            app.calendar_date = now.date_naive();
            return recharger(app);
        }
        Message::ToggleTheme => {
            // La bascule rapide et le sélecteur à trois valeurs pilotent la même chose : la
            // bascule fixe donc explicitement la préférence, faute de quoi le sélecteur
            // afficherait « Système » alors que l'utilisateur vient de choisir la main.
            app.is_dark = !app.is_dark;
            app.data.settings.theme = if app.is_dark {
                crate::modules::settings::model::ThemePref::Dark
            } else {
                crate::modules::settings::model::ThemePref::Light
            };
            app.settings_form.draft.theme = app.data.settings.theme;
            let Some(backend) = app.backend.clone() else {
                app.notify_failure("La préférence de thème n'a pas pu être enregistrée.");
                return Task::none();
            };
            let settings = app.data.settings.clone();
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        backend
                            .settings
                            .persist(&settings)
                            .map_err(|error| error.to_string())
                    })
                    .await
                    .unwrap_or_else(|error| Err(format!("Opération interrompue : {error}")))
                },
                Message::ThemePersisted,
            );
        }
        Message::ThemePersisted(result) => match result {
            Ok(settings) => {
                app.data.settings = settings.clone();
                app.settings_form.draft.theme = settings.theme;
            }
            Err(error) => app.notify_failure(format!("Thème non enregistré : {error}")),
        },
        Message::Tick => {
            if app.ai_is_running {
                app.ai_elapsed_seconds = app.ai_elapsed_seconds.saturating_add(1);
            }
        }
        Message::CheckUpdate => {
            // Fabrique centralisée : `reqwest::Client::new()` n'applique aucun délai et suit
            // les redirections jusqu'à 10 sauts. Un serveur qui accepte la connexion sans
            // jamais répondre faisait attendre indéfiniment, sans message ni moyen d'annuler.
            let client = crate::shared::http::client();
            let current =
                Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or_else(|_| Version::new(0, 0, 0));
            return Task::perform(
                async move {
                    crate::core::updater::check(&client, &current)
                        .await
                        .map_err(|error| error.to_string())
                },
                Message::UpdateChecked,
            );
        }
        Message::UpdateChecked(result) => match result {
            Ok(Some(info)) => {
                app.notify(
                    NotificationKind::Info,
                    format!("Candilog {} est disponible.", info.version),
                );
                app.available_update = Some(info);
            }
            Ok(None) => app.notify(NotificationKind::Info, "Candilog est à jour."),
            Err(error) => app.notify_failure(format!("Vérification impossible : {error}")),
        },
        Message::ClearNotification => {
            app.notification = None;
            app.notification_shown_at = None;
        }
        Message::NotificationCountdown => {
            let expire = app
                .notification_shown_at
                .is_some_and(|pose| pose.elapsed() >= crate::app::state::DURATION_AFFICHAGE_TOAST);
            if expire {
                app.notification = None;
                app.notification_shown_at = None;
            }
        }
        Message::OpenDialog(dialog) => {
            app.dialog = Some(dialog);
            app.editing_id = None;
            app.company_option_search.clear();
            app.candidate_option_search.clear();
            app.contact_option_search.clear();
            app.company_option_page = 1;
            app.candidate_option_page = 1;
            app.contact_option_page = 1;
            match dialog {
                Dialog::Entreprise => app.entreprise_form = EntrepriseForm::default(),
                Dialog::Contact => app.contact_form = ContactForm::default(),
                Dialog::Candidature => app.candidature_form = CandidatureForm::default(),
                Dialog::Entretien => app.entretien_form = EntretienForm::default(),
                Dialog::Relance => app.relance_form = RelanceForm::default(),
                Dialog::Profil(_) => {
                    app.profile_personal_form = app.data.profile.personal.clone();
                    app.profile_draft = app.data.profile.clone();
                    app.profile_summary_editor = iced::widget::text_editor::Content::with_text(
                        app.data
                            .profile
                            .personal
                            .summary
                            .as_deref()
                            .unwrap_or_default(),
                    );
                    app.profile_skills_form.clear();
                }
                Dialog::ProfileImport => {}
                Dialog::DeleteCandidature(_)
                | Dialog::DeleteEntreprise(_)
                | Dialog::DeleteContact(_)
                | Dialog::DeleteEntretien(_)
                | Dialog::DeleteRelance(_)
                | Dialog::DeleteCv(_)
                | Dialog::ImportBackup
                | Dialog::ResetDatabase
                | Dialog::ResetAiCache
                | Dialog::CandidatureDetail(_) => {}
            }
            if matches!(
                dialog,
                Dialog::Contact | Dialog::Candidature | Dialog::Entretien | Dialog::Relance
            ) {
                return recharger(app);
            }
        }
        Message::CloseDialog => {
            if app.write_in_progress {
                return Task::none();
            }
            app.dialog = None;
            app.editing_id = None;
            // `selected_contact` n'est plus effacé ici : `CloseDialog` sert les six modales,
            // et l'effacement — ajouté pour la seule fermeture de la fiche contact —
            // s'appliquait à tous les cas. Voir `Message::CloseContactCard`.
        }
        Message::CloseContactCard => app.selected_contact = None,
        Message::DismissTopLayer => {
            if app.write_in_progress {
                return Task::none();
            }
            // Échap ferme ce qui est ouvert, et rien d'autre. Intercepté globalement et sans
            // condition, il désélectionnait le contact affiché dans l'inspecteur du Réseau —
            // et lui seul, ni la candidature, ni l'entreprise, ni le CV sélectionnés — alors
            // même qu'aucun dialogue n'était ouvert.
            if app.date_picker.is_some() {
                app.date_picker = None;
            } else if app.dialog.is_some() {
                app.dialog = None;
                app.editing_id = None;
            } else if app.selected_contact.is_some() {
                app.selected_contact = None;
            }
        }
        _ => unreachable!("message non routé"),
    }
    Task::none()
}

/// Identifiant du champ de recherche de l'écran courant, ciblé par `Ctrl+F`.
pub const SEARCH_FIELD_ID: &str = "candilog-search";

fn optional(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// Seuil de déplacement (px) au-delà duquel un appui sur une carte devient un glisser.
const DRAG_THRESHOLD: f32 = 5.0;

/// Vrai si le curseur s'est déplacé au-delà du seuil de glisser depuis l'origine.
fn depasse_le_seuil(origin: iced::Point, cursor: iced::Point) -> bool {
    (cursor.x - origin.x).hypot(cursor.y - origin.y) > DRAG_THRESHOLD
}

#[cfg(test)]
#[path = "tests/update/mod.rs"]
mod tests;
