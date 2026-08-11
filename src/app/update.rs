//! Traitement des messages Iced.

use super::capture::save_review_screenshot;
use super::commandes::{ecrire, finish_submit, notifier_le_bureau, recharger};
use super::export::export_candidatures;
use super::message::{LetterStreamEvent, UpdateDownloadEvent};
use super::state::{
    CandidatureForm, ContactForm, Dialog, EntrepriseForm, EntretienForm, NotificationKind,
    RelanceForm,
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

/// Met à jour l'état applicatif.
pub fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Noop => {}
        Message::WriteFinished(result, succes) => {
            finish_submit(app, result, succes);
            return recharger(app);
        }
        Message::DataLoaded(data, echecs) => app.appliquer_instantane(*data, &echecs),
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
            if route == crate::navigation::Route::Parametres {
                // Le brouillon repart de l'état persisté : quitter l'écran sans enregistrer
                // ne doit pas laisser de modification en mémoire.
                app.settings_form =
                    crate::app::state::SettingsForm::from_settings(&app.data.settings);
            }
            app.route = route;
            app.search.clear();
        }
        Message::Reload => return recharger(app),
        Message::SearchChanged(value) => app.search = value,
        Message::CandidateViewChanged(mode) => app.candidate_view = mode,
        Message::PreviousMonth => match app.calendar_view {
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
        },
        Message::NextMonth => match app.calendar_view {
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
        },
        Message::CurrentMonth => {
            let now = Local::now();
            app.calendar_year = now.year();
            app.calendar_month = now.month();
            app.calendar_date = now.date_naive();
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
        }
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
        Message::ClearNotification => app.notification = None,
        Message::OpenDialog(dialog) => {
            app.dialog = Some(dialog);
            app.editing_id = None;
            match dialog {
                Dialog::Entreprise => app.entreprise_form = EntrepriseForm::default(),
                Dialog::Contact => app.contact_form = ContactForm::default(),
                Dialog::Candidature => {
                    app.candidature_form = CandidatureForm::default();
                    app.candidature_form.entreprise_id =
                        app.data.entreprises.first().map(|item| item.id);
                }
                Dialog::Entretien => {
                    app.entretien_form = EntretienForm::default();
                    app.entretien_form.candidature_id =
                        app.data.candidatures.first().map(|item| item.id);
                }
                Dialog::Relance => {
                    app.relance_form = RelanceForm::default();
                    app.relance_form.candidature_id =
                        app.data.candidatures.first().map(|item| item.id);
                }
                Dialog::Profil => {
                    app.profile_personal_form = app.data.profile.personal.clone();
                    app.profile_skills_form = app
                        .data
                        .profile
                        .skills
                        .iter()
                        .map(|skill| skill.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                }
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
        }
        Message::CloseDialog => {
            app.dialog = None;
            app.editing_id = None;
            // `selected_contact` n'est plus effacé ici : `CloseDialog` sert les six modales,
            // et l'effacement — ajouté pour la seule fermeture de la fiche contact —
            // s'appliquait à tous les cas. Voir `Message::CloseContactCard`.
        }
        Message::CloseContactCard => app.selected_contact = None,
        Message::DismissTopLayer => {
            // Échap ferme ce qui est ouvert, et rien d'autre. Intercepté globalement et sans
            // condition, il désélectionnait le contact affiché dans l'inspecteur du Réseau —
            // et lui seul, ni la candidature, ni l'entreprise, ni le CV sélectionnés — alors
            // même qu'aucun dialogue n'était ouvert.
            if app.dialog.is_some() {
                app.dialog = None;
                app.editing_id = None;
            } else if app.selected_contact.is_some() {
                app.selected_contact = None;
            }
        }
        Message::EntrepriseNomChanged(value) => app.entreprise_form.nom = value,
        Message::EntrepriseSecteurChanged(value) => app.entreprise_form.secteur = value,
        Message::EntrepriseTypeChanged(value) => app.entreprise_form.type_ = value,
        Message::EntrepriseSiteChanged(value) => app.entreprise_form.site_web = value,
        Message::EntrepriseVilleChanged(value) => app.entreprise_form.ville = value,
        Message::EntrepriseAdresseChanged(value) => app.entreprise_form.adresse = value,
        Message::EntrepriseNotesChanged(value) => app.entreprise_form.notes = value,
        Message::SubmitEntreprise => {
            let input = NouvelleEntreprise {
                nom: app.entreprise_form.nom.clone(),
                secteur: optional(&app.entreprise_form.secteur),
                type_: optional(&app.entreprise_form.type_),
                site_web: optional(&app.entreprise_form.site_web),
                ville: optional(&app.entreprise_form.ville),
                adresse: optional(&app.entreprise_form.adresse),
                notes: optional(&app.entreprise_form.notes),
            };
            let edition = app.editing_id;
            return ecrire(app, "Entreprise enregistrée.", move |backend| {
                edition
                    .map_or_else(
                        || backend.entreprises.creer(&input),
                        |id| backend.entreprises.modifier(id, &input),
                    )
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            });
        }
        Message::ContactPrenomChanged(value) => app.contact_form.prenom = value,
        Message::ContactNomChanged(value) => app.contact_form.nom = value,
        Message::ContactPosteChanged(value) => app.contact_form.poste = value,
        Message::ContactEmailChanged(value) => app.contact_form.email = value,
        Message::ContactTelephoneChanged(value) => app.contact_form.telephone = value,
        Message::ContactLinkedinChanged(value) => app.contact_form.linkedin = value,
        Message::ContactNotesChanged(value) => app.contact_form.notes = value,
        Message::ContactEntrepriseChanged(value) => app.contact_form.entreprise_id = value,
        Message::SubmitContact => {
            let input = NouveauContact {
                entreprise_id: app.contact_form.entreprise_id,
                prenom: app.contact_form.prenom.clone(),
                nom: app.contact_form.nom.clone(),
                poste: optional(&app.contact_form.poste),
                email: optional(&app.contact_form.email),
                telephone: optional(&app.contact_form.telephone),
                linkedin: optional(&app.contact_form.linkedin),
                notes: optional(&app.contact_form.notes),
            };
            let edition = app.editing_id;
            return ecrire(app, "Contact enregistré.", move |backend| {
                edition
                    .map_or_else(
                        || backend.contacts.creer(&input),
                        |id| backend.contacts.modifier(id, &input),
                    )
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            });
        }
        Message::CandidaturePosteChanged(value) => app.candidature_form.poste = value,
        Message::CandidatureEntrepriseChanged(value) => {
            app.candidature_form.entreprise_id = Some(value)
        }
        Message::CandidatureContratChanged(value) => app.candidature_form.type_contrat = value,
        Message::CandidatureStatutChanged(value) => app.candidature_form.statut = value,
        Message::CandidatureDateChanged(value) => app.candidature_form.date_envoi = value,
        Message::CandidatureLienChanged(value) => app.candidature_form.lien_offre = value,
        Message::CandidatureNotesChanged(value) => app.candidature_form.notes = value,
        Message::SubmitCandidature => {
            let Some(entreprise_id) = app.candidature_form.entreprise_id else {
                app.notify(NotificationKind::Warning, "Sélectionnez une entreprise.");
                return Task::none();
            };
            let date_envoi = match ui_format::date_to_storage(&app.candidature_form.date_envoi) {
                Ok(date) => date,
                Err(error) => {
                    app.notify(
                        NotificationKind::Warning,
                        format!("Date d'envoi invalide. {error}"),
                    );
                    return Task::none();
                }
            };
            let input = NouvelleCandidature {
                poste: app.candidature_form.poste.clone(),
                entreprise_id,
                type_contrat: app.candidature_form.type_contrat,
                statut: app.candidature_form.statut,
                date_envoi,
                lien_offre: optional(&app.candidature_form.lien_offre),
                notes: optional(&app.candidature_form.notes),
            };
            let edition = app.editing_id;
            return ecrire(app, "Candidature enregistrée.", move |backend| {
                edition
                    .map_or_else(
                        || backend.candidatures.creer(&input),
                        |id| backend.candidatures.modifier(id, &input),
                    )
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            });
        }
        Message::MoveCandidature(id, status) => {
            let Some(backend) = app.backend.clone() else {
                app.notify_failure("La base Candilog n'est pas disponible.");
                return Task::none();
            };
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        backend
                            .candidatures
                            .changer_statut(id, status)
                            .map(|_| ())
                            .map_err(|error| error.to_string())
                    })
                    .await
                    .unwrap_or_else(|error| Err(format!("Opération interrompue : {error}")))
                },
                Message::CandidatureStatusUpdated,
            );
        }
        Message::CandidatureStatusUpdated(result) => match result {
            Ok(()) => {
                app.notify_success("Statut de la candidature mis à jour.");
                return recharger(app);
            }
            Err(error) => app.notify_failure(error),
        },
        Message::EntretienCandidatureChanged(value) => {
            app.entretien_form.candidature_id = Some(value)
        }
        Message::EntretienContactChanged(value) => app.entretien_form.contact_id = value,
        Message::EntretienDateChanged(value) => app.entretien_form.date_entretien = value,
        Message::EntretienTypeChanged(value) => app.entretien_form.type_entretien = value,
        Message::EntretienLieuChanged(value) => app.entretien_form.lieu = value,
        Message::EntretienNotesChanged(value) => app.entretien_form.notes = value,
        Message::EntretienCompteRenduChanged(action) => {
            app.entretien_form.compte_rendu.perform(action);
        }
        Message::SubmitEntretien => {
            let Some(candidature_id) = app.entretien_form.candidature_id else {
                app.notify(NotificationKind::Warning, "Sélectionnez une candidature.");
                return Task::none();
            };
            let date_entretien =
                match ui_format::datetime_to_storage(&app.entretien_form.date_entretien) {
                    Ok(date) => date,
                    Err(error) => {
                        app.notify(
                            NotificationKind::Warning,
                            format!("Date d'entretien invalide. {error}"),
                        );
                        return Task::none();
                    }
                };
            let compte_rendu = app.entretien_form.compte_rendu.text();
            let input = NouvelEntretien {
                candidature_id,
                contact_id: app.entretien_form.contact_id,
                date_entretien,
                type_entretien: app.entretien_form.type_entretien,
                lieu: optional(&app.entretien_form.lieu),
                notes: optional(&app.entretien_form.notes),
                compte_rendu: optional(&compte_rendu),
            };
            let edition = app.editing_id;
            return ecrire(app, "Entretien enregistré.", move |backend| {
                edition
                    .map_or_else(
                        || backend.entretiens.creer(&input),
                        |id| backend.entretiens.modifier(id, &input),
                    )
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            });
        }
        Message::RelanceCandidatureChanged(value) => app.relance_form.candidature_id = Some(value),
        Message::RelanceDateChanged(value) => app.relance_form.date_relance = value,
        Message::RelanceTypeChanged(value) => app.relance_form.type_relance = value,
        Message::RelanceNotesChanged(value) => app.relance_form.notes = value,
        Message::SubmitRelance => {
            let Some(candidature_id) = app.relance_form.candidature_id else {
                app.notify(NotificationKind::Warning, "Sélectionnez une candidature.");
                return Task::none();
            };
            let date_relance = match ui_format::date_to_storage(&app.relance_form.date_relance) {
                Ok(date) => date,
                Err(error) => {
                    app.notify(
                        NotificationKind::Warning,
                        format!("Date de relance invalide. {error}"),
                    );
                    return Task::none();
                }
            };
            let input = NouvelleRelance {
                candidature_id,
                date_relance,
                type_relance: app.relance_form.type_relance.clone(),
                notes: optional(&app.relance_form.notes),
            };
            let edition = app.editing_id;
            return ecrire(app, "Relance enregistrée.", move |backend| {
                edition
                    .map_or_else(
                        || backend.relances.creer(&input),
                        |id| backend.relances.modifier(id, &input),
                    )
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            });
        }
        Message::ExportCandidatures => {
            let rows = app.filtered_candidates().into_iter().cloned().collect();
            return Task::perform(export_candidatures(rows), Message::CandidaturesExported);
        }
        Message::CandidaturesExported(result) => match result {
            Ok(path) => app.notify_success(format!("Export créé : {}", path.display())),
            Err(error) => app.notify_failure(error),
        },
        Message::OfferEditorAction(action) => app.offer_editor.perform(action),
        Message::AnalyzeOffer => {
            let Some(backend) = app.backend.clone() else {
                app.notify_failure("La base Candilog n'est pas disponible.");
                return Task::none();
            };
            let offer = app.offer_editor.text();
            app.offer_analysis = None;
            app.cv_generation = None;
            let token = tokio_util::sync::CancellationToken::new();
            let sequence = app.commencer_operation_ia(token.clone());
            return Task::perform(
                async move {
                    tokio::select! {
                        result = crate::modules::ia::service::analyze_offer(&backend, offer) => {
                            result.map_err(|error| error.to_string())
                        }
                        () = token.cancelled() => Err("Génération annulée".into()),
                    }
                },
                move |result| Message::OfferAnalyzed(result, sequence),
            );
        }
        Message::OfferAnalyzed(result, sequence) => {
            if !app.terminer_operation_ia(sequence) {
                return Task::none();
            }
            match result {
                Ok(analysis) => {
                    app.offer_analysis = Some(analysis);
                    app.notify_success(format!("Offre analysée en {} s.", app.ai_elapsed_seconds));
                }
                Err(error) => app.notify_failure(error),
            }
        }
        Message::GenerateCv => {
            let (Some(backend), Some(analysis)) = (app.backend.clone(), app.offer_analysis.clone())
            else {
                app.notify(NotificationKind::Warning, "Analysez d'abord une offre.");
                return Task::none();
            };
            let token = tokio_util::sync::CancellationToken::new();
            let sequence = app.commencer_operation_ia(token.clone());
            return Task::perform(
                async move {
                    crate::modules::ia::service::generate_cv(
                        &backend,
                        analysis.parsed,
                        analysis.score,
                        token,
                    )
                    .await
                    .map_err(|error| error.to_string())
                },
                move |result| Message::CvGenerated(result, sequence),
            );
        }
        Message::CvGenerated(result, sequence) => {
            if !app.terminer_operation_ia(sequence) {
                return Task::none();
            }
            match result {
                Ok(generation) => {
                    app.recommendation_states = vec![
                        super::state::RecommendationStatus::Pending;
                        generation.analysis.recommandations.len()
                    ];
                    app.cv_generation = Some(generation);
                    let corps = format!("CV généré en {} s.", app.ai_elapsed_seconds);
                    app.notify_success(corps.clone());
                    return notifier_le_bureau(corps);
                }
                Err(error) => app.notify_failure(error),
            }
        }
        Message::CancelAi => {
            if let Some(token) = &app.ai_cancellation {
                token.cancel();
                app.notify(NotificationKind::Info, "Annulation demandée…");
            }
        }
        Message::SelectImportPdf => {
            return Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Sélectionner un CV PDF")
                        .add_filter("PDF", &["pdf"])
                        .pick_file()
                        .await
                        .map(|file| file.path().to_path_buf())
                },
                Message::ImportPdfSelected,
            );
        }
        Message::ImportPdfSelected(path) => app.import_pdf_path = path,
        Message::ImportOfferEditorAction(action) => app.import_offer_editor.perform(action),
        Message::AnalyzeImportedCv => {
            let (Some(backend), Some(path)) = (app.backend.clone(), app.import_pdf_path.clone())
            else {
                app.notify(
                    NotificationKind::Warning,
                    "Sélectionnez un PDF avant l'analyse.",
                );
                return Task::none();
            };
            let offer = app.import_offer_editor.text();
            let token = tokio_util::sync::CancellationToken::new();
            let sequence = app.commencer_operation_ia(token.clone());
            app.imported_cv_analysis = None;
            return Task::perform(
                async move {
                    crate::modules::ia::service::analyze_imported_cv(&backend, &path, offer, token)
                        .await
                        .map_err(|error| error.to_string())
                },
                move |result| Message::ImportedCvAnalyzed(result, sequence),
            );
        }
        Message::ImportedCvAnalyzed(result, sequence) => {
            if !app.terminer_operation_ia(sequence) {
                return Task::none();
            }
            match result {
                Ok(analysis) => {
                    app.imported_cv_analysis = Some(analysis);
                    let corps = format!("CV analysé en {} s.", app.ai_elapsed_seconds);
                    app.notify_success(corps.clone());
                    return notifier_le_bureau(corps);
                }
                Err(error) => app.notify_failure(error),
            }
        }
        Message::LetterCompanyChanged(value) => app.letter_company = value,
        Message::LetterJobTitleChanged(value) => app.letter_job_title = value,
        Message::LetterEditorAction(action) => app.letter_editor.perform(action),
        Message::LetterToneChanged(value) => app.letter_tone = value,
        Message::LetterLengthChanged(value) => app.letter_length = value,
        Message::GenerateLetter => {
            let Some(backend) = app.backend.clone() else {
                app.notify_failure("La base Candilog n'est pas disponible.");
                return Task::none();
            };
            let token = tokio_util::sync::CancellationToken::new();
            let sequence = app.commencer_operation_ia(token.clone());
            app.letter_output.clear();
            let request = crate::modules::ia::cv_model::LetterGenerationRequest {
                source: "job_offer".into(),
                generation_id: uuid::Uuid::new_v4().to_string(),
                company_name: optional(&app.letter_company),
                job_title: optional(&app.letter_job_title),
                job_description: optional(&app.letter_editor.text()),
                tone: Some(app.letter_tone.clone()),
                length: Some(app.letter_length.clone()),
                ..crate::modules::ia::cv_model::LetterGenerationRequest::default()
            };
            let stream = iced::stream::channel(100, move |mut sender| async move {
                let mut chunk_sender = sender.clone();
                let mut on_chunk = move |chunk: String| {
                    let _ = chunk_sender.try_send(LetterStreamEvent::Chunk(chunk));
                };
                let result = crate::modules::ia::service::generate_cover_letter(
                    &backend,
                    &request,
                    token,
                    &mut on_chunk,
                )
                .await
                .map_err(|error| error.to_string());
                let _ = sender.send(LetterStreamEvent::Finished(result)).await;
            });
            return Task::run(stream, move |event| Message::LetterStream(event, sequence));
        }
        Message::LetterStream(event, sequence) => match event {
            LetterStreamEvent::Chunk(chunk) => {
                // Un fragment d'une génération abandonnée ne doit pas polluer l'éditeur.
                if app.ai_sequence == sequence {
                    app.letter_output.push_str(&chunk);
                }
            }
            LetterStreamEvent::Finished(result) => {
                if !app.terminer_operation_ia(sequence) {
                    return Task::none();
                }
                match result {
                    Ok(letter) => {
                        app.letter_output = letter;
                        let corps = format!("Lettre générée en {} s.", app.ai_elapsed_seconds);
                        app.notify_success(corps.clone());
                        return notifier_le_bureau(corps);
                    }
                    Err(error) => app.notify_failure(error),
                }
            }
        },
        // Les huit messages d'édition écrivent dans le **brouillon**, jamais dans
        // l'instantané : seul `SettingsSaved(Ok(settings))` recopie la valeur réellement
        // persistée.
        Message::SettingsProviderChanged(provider) => {
            app.settings_form.draft.llm.provider = provider;
            app.available_models.clear();
            if matches!(
                app.settings_form.draft.llm.provider,
                crate::shared::llm::ProviderKind::Ollama
            ) && app.settings_form.draft.llm.endpoint.is_none()
            {
                app.settings_form.draft.llm.endpoint = Some("http://localhost:11434".into());
            }
        }
        Message::SettingsModelChanged(value) => app.settings_form.draft.llm.model = value,
        Message::RefreshLlmModels => {
            let config = app.settings_form.draft.llm.clone();
            app.provider_health = crate::ui::components::runtime_status::Health::Checking;
            return Task::perform(
                async move {
                    let pin = crate::shared::llm::validate_llm_endpoint(&config)
                        .await
                        .map_err(|error| error.to_string())?;
                    crate::modules::ia::factory::build_provider_pinned(&config, pin)
                        .list_models()
                        .await
                        .map_err(|error| error.to_string())
                },
                Message::LlmModelsLoaded,
            );
        }
        Message::LlmModelsLoaded(result) => match result {
            Ok(mut models) => {
                models.sort();
                models.dedup();
                if !app.settings_form.draft.llm.model.trim().is_empty()
                    && !models.contains(&app.settings_form.draft.llm.model)
                {
                    models.insert(0, app.settings_form.draft.llm.model.clone());
                }
                app.available_models = models;
                app.provider_health = crate::ui::components::runtime_status::Health::Ok;
                app.notify_success("Liste des modèles actualisée.");
            }
            Err(error) => {
                app.provider_health = crate::ui::components::runtime_status::Health::Error;
                app.notify_failure(format!("Modèles indisponibles : {error}"));
            }
        },
        Message::SettingsEndpointChanged(value) => {
            app.settings_form.draft.llm.endpoint = optional(&value);
        }
        Message::SettingsApiKeyChanged(value) => {
            app.settings_form.draft.llm.api_key = optional(&value);
        }
        Message::SettingsTemperatureChanged(value) => {
            app.settings_form.draft.llm.temperature = value;
        }
        Message::SettingsModeChanged(value) => app.settings_form.draft.llm.mode = value,
        Message::SettingsThemeChanged(value) => {
            // Le thème est la seule exception assumée : l'aperçu doit être immédiat.
            app.is_dark = crate::core::theme_systeme::resoudre(value, app.system_dark, app.is_dark);
            app.settings_form.draft.theme = value;
            // Le système a pu changer d'avis depuis le démarrage : on redemande.
            if matches!(value, crate::modules::settings::model::ThemePref::System) {
                return Task::perform(
                    crate::core::theme_systeme::detecter(),
                    Message::SystemThemeDetected,
                );
            }
        }
        Message::SystemThemeDetected(sombre) => {
            app.system_dark = sombre;
            app.is_dark = crate::core::theme_systeme::resoudre(
                app.data.settings.theme,
                app.system_dark,
                app.is_dark,
            );
        }
        Message::SaveSettings => {
            let Some(backend) = app.backend.clone() else {
                app.notify_failure("La base Candilog n'est pas disponible.");
                return Task::none();
            };
            let settings = app.settings_form.draft.clone();
            return Task::perform(
                async move {
                    backend
                        .update_secure_settings(settings)
                        .await
                        .map_err(|error| error.to_string())
                },
                Message::SettingsSaved,
            );
        }
        Message::SettingsSaved(result) => match result {
            Ok(settings) => {
                app.settings_form = crate::app::state::SettingsForm::from_settings(&settings);
                app.data.settings = settings;
                app.notify_success("Paramètres enregistrés.");
                return Task::done(Message::ProbeProviderHealth);
            }
            Err(error) => app.notify_failure(error),
        },
        Message::TestLlmConnection => {
            // Teste ce que l'utilisateur vient de saisir, pas ce qui est déjà enregistré.
            let config = app.settings_form.draft.llm.clone();
            app.provider_health = crate::ui::components::runtime_status::Health::Checking;
            return Task::perform(
                async move {
                    let pin = crate::shared::llm::validate_llm_endpoint(&config)
                        .await
                        .map_err(|error| error.to_string())?;
                    crate::modules::ia::factory::build_provider_pinned(&config, pin)
                        .health_check()
                        .await
                        .map_err(|error| error.to_string())
                },
                Message::LlmConnectionTested,
            );
        }
        Message::LlmConnectionTested(result) => match result {
            Ok(()) => {
                app.provider_health = crate::ui::components::runtime_status::Health::Ok;
                app.notify_success("Connexion IA opérationnelle.");
            }
            Err(error) => {
                app.provider_health = crate::ui::components::runtime_status::Health::Error;
                app.notify_failure(format!("Connexion IA impossible : {error}"));
            }
        },
        Message::OpenAuthorWebsite => {
            return Task::perform(
                async {
                    tokio::task::spawn_blocking(|| {
                        crate::core::external::open_https("https://www.alexandrebouttier.fr")
                    })
                    .await
                    .unwrap_or_else(|error| Err(format!("Ouverture interrompue : {error}")))
                },
                Message::AuthorWebsiteOpened,
            );
        }
        Message::AuthorWebsiteOpened(result) => {
            if let Err(error) = result {
                app.notify_failure(error);
            }
        }
        Message::ExportBackup => {
            let Some(backend) = app.backend.clone() else {
                app.notify_failure("La base Candilog n'est pas disponible.");
                return Task::none();
            };
            return Task::perform(
                async move {
                    let Some(file) = rfd::AsyncFileDialog::new()
                        .set_title("Exporter la base Candilog")
                        .set_file_name("candilog-backup.sqlite")
                        .add_filter("SQLite", &["sqlite"])
                        .save_file()
                        .await
                    else {
                        return Err("Export annulé.".into());
                    };
                    let path = file.path().to_path_buf();
                    crate::core::backup::export(&backend.sqlite, &path)
                        .map_err(|error| error.to_string())?;
                    Ok(path)
                },
                Message::BackupExported,
            );
        }
        Message::BackupExported(result) => match result {
            Ok(path) => app.notify_success(format!("Backup créé : {}", path.display())),
            Err(error) => app.notify_failure(error),
        },
        Message::CandidatePressed(id) => {
            app.press_candidate = Some(id);
            app.press_origin = None;
            app.dragging_candidate = None;
            app.drag_target_status = None;
        }
        Message::CandidateMoved(point) => {
            let Some(id) = app.press_candidate else {
                return Task::none();
            };
            match app.press_origin {
                None => app.press_origin = Some(point),
                Some(origin) if depasse_le_seuil(origin, point) => {
                    app.dragging_candidate = Some(id);
                    app.press_candidate = None;
                }
                Some(_) => {}
            }
        }
        Message::CandidateReleased => {
            if let Some(id) = app.dragging_candidate.take() {
                app.press_candidate = None;
                app.press_origin = None;
                app.hovered_card = None;
                let target = app.drag_target_status.take();
                if let Some(status) = target {
                    return ecrire(
                        app,
                        "Statut mis à jour par glisser-déposer.",
                        move |backend| {
                            backend
                                .candidatures
                                .changer_statut(id, status)
                                .map(|_| ())
                                .map_err(|error| error.to_string())
                        },
                    );
                }
                return Task::none();
            }
            if let Some(id) = app.press_candidate.take() {
                app.press_origin = None;
                app.drag_target_status = None;
                return Task::done(Message::OpenDialog(Dialog::CandidatureDetail(id)));
            }
            return Task::none();
        }
        Message::CandidateCardHovered(id) => app.hovered_card = Some(id),
        Message::CandidateCardExited => app.hovered_card = None,
        Message::CandidateDragHovered(status) => {
            if app.dragging_candidate.is_some() {
                app.drag_target_status = Some(status);
            }
        }
        Message::CandidateDropped(status) => {
            app.press_candidate = None;
            app.press_origin = None;
            app.hovered_card = None;
            let Some(id) = app.dragging_candidate.take() else {
                return Task::none();
            };
            app.drag_target_status = None;
            return ecrire(
                app,
                "Statut mis à jour par glisser-déposer.",
                move |backend| {
                    backend
                        .candidatures
                        .changer_statut(id, status)
                        .map(|_| ())
                        .map_err(|error| error.to_string())
                },
            );
        }
        Message::CandidateDragCancelled => {
            app.press_candidate = None;
            app.press_origin = None;
            app.dragging_candidate = None;
            app.drag_target_status = None;
        }
        Message::CalendarViewChanged(view) => app.calendar_view = view,
        Message::CalendarDateSelected(date) => {
            app.calendar_date = date;
            app.calendar_year = date.year();
            app.calendar_month = date.month();
            app.calendar_view = super::message::CalendarView::Day;
        }
        Message::ProfileFirstNameChanged(value) => app.profile_personal_form.first_name = value,
        Message::ProfileLastNameChanged(value) => app.profile_personal_form.last_name = value,
        Message::ProfileEmailChanged(value) => app.profile_personal_form.email = value,
        Message::ProfilePhoneChanged(value) => app.profile_personal_form.phone = optional(&value),
        Message::ProfileCityChanged(value) => app.profile_personal_form.city = optional(&value),
        Message::ProfileHeadlineChanged(value) => {
            app.profile_personal_form.headline = optional(&value)
        }
        Message::ProfileSummaryChanged(value) => {
            app.profile_personal_form.summary = optional(&value)
        }
        Message::ProfileSkillsChanged(value) => app.profile_skills_form = value,
        Message::SubmitProfile => {
            let mut profile = app.data.profile.clone();
            profile.personal = app.profile_personal_form.clone();
            profile.skills = app
                .profile_skills_form
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(|name| crate::shared::profile::Skill { name: name.into() })
                .collect();
            return ecrire(app, "Profil enregistré.", move |backend| {
                backend
                    .profil
                    .update(&profile)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            });
        }
        Message::DownloadUpdate => {
            let Some(update) = app.available_update.clone() else {
                app.notify(NotificationKind::Info, "Aucune mise à jour disponible.");
                return Task::none();
            };
            app.update_progress = Some(0);
            app.verified_update_path = None;
            let dossier = app
                .paths
                .as_ref()
                .map_or_else(std::env::temp_dir, |paths| paths.data_dir.clone());
            let stream = iced::stream::channel(32, move |mut sender| async move {
                let client = crate::shared::http::download_client();
                let mut progress_sender = sender.clone();
                let result =
                    crate::core::updater::download_verified(&client, &update, &dossier, |value| {
                        let _ = progress_sender.try_send(UpdateDownloadEvent::Progress(value));
                    })
                    .await
                    .map_err(|error| error.to_string());
                let _ = sender.send(UpdateDownloadEvent::Finished(result)).await;
            });
            return Task::run(stream, Message::UpdateDownload);
        }
        Message::UpdateDownload(event) => match event {
            UpdateDownloadEvent::Progress(value) => app.update_progress = Some(value),
            UpdateDownloadEvent::Finished(result) => match result {
                Ok(path) => {
                    app.update_progress = Some(100);
                    app.verified_update_path = Some(path.clone());
                    let corps = format!(
                        "Mise à jour téléchargée et signature vérifiée. Le paquet est dans \
                         le dossier « {} » de vos données Candilog ; installez-le comme vous \
                         le feriez pour toute application de votre système.",
                        crate::core::updater::DOSSIER_MISES_A_JOUR
                    );
                    app.notify_success(corps.clone());
                    return notifier_le_bureau(corps);
                }
                Err(error) => {
                    app.update_progress = None;
                    app.notify_failure(error);
                }
            },
        },
        Message::CandidateFilterStatus(value) => app.candidate_filters.status = value,
        Message::CandidateFilterContract(value) => app.candidate_filters.contract = value,
        Message::CandidateFilterCompany(value) => app.candidate_filters.company_id = value,
        Message::CandidateFilterCity(value) => app.candidate_filters.city = value,
        Message::CandidateFilterPosition(value) => app.candidate_filters.position = value,
        Message::CandidateFilterDateFrom(value) => app.candidate_filters.date_from = value,
        Message::CandidateFilterDateTo(value) => app.candidate_filters.date_to = value,
        Message::ResetCandidateFilters => {
            app.candidate_filters = super::state::CandidateFilters::default();
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
                    notes: item.notes.clone().unwrap_or_default(),
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
                    notes: item.notes.clone().unwrap_or_default(),
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
                    notes: item.notes.clone().unwrap_or_default(),
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
        Message::CvVersionNameChanged(value) => app.cv_version_name = value,
        Message::SaveGeneratedCv => {
            let (Some(backend), Some(generation)) =
                (app.backend.as_ref(), app.cv_generation.as_ref())
            else {
                app.notify(
                    NotificationKind::Warning,
                    "Générez un CV avant de le sauvegarder.",
                );
                return Task::none();
            };
            let content = serde_json::json!({
                "cv": generation.cv,
                "analysis": generation.analysis,
                "offer": app.offer_analysis.as_ref().map(|analysis| &analysis.parsed),
                "matchScore": app.offer_analysis.as_ref().map(|analysis| &analysis.score),
                "personal": app.data.profile.personal,
            });
            match backend.cv.save(&app.cv_version_name, &content) {
                Ok(_) => {
                    app.notify_success("Version de CV sauvegardée.");
                    return recharger(app);
                }
                Err(error) => app.notify_error(&error),
            }
        }
        Message::LoadCvVersion(id) => {
            let Some(backend) = app.backend.as_ref() else {
                app.notify_failure("La base Candilog n'est pas disponible.");
                return Task::none();
            };
            match backend.cv.load(id) {
                Ok(version) => {
                    let cv = serde_json::from_value(version.content["cv"].clone());
                    let analysis = serde_json::from_value(version.content["analysis"].clone());
                    match (cv, analysis) {
                        (Ok(cv), Ok(analysis)) => {
                            app.cv_generation =
                                Some(crate::modules::ia::cv_model::CvGeneration { cv, analysis });
                            app.recommendation_states =
                                app.cv_generation
                                    .as_ref()
                                    .map_or_else(Vec::new, |generation| {
                                        vec![
                                            super::state::RecommendationStatus::Pending;
                                            generation.analysis.recommandations.len()
                                        ]
                                    });
                            app.cv_version_name = version.name;
                            app.route = crate::navigation::Route::CvGenerator;
                        }
                        _ => app.notify_failure("Cette version de CV est illisible."),
                    }
                }
                Err(error) => app.notify_error(&error),
            }
        }
        Message::ExportGeneratedCvPdf => {
            let Some(generation) = app.cv_generation.as_ref() else {
                app.notify(
                    NotificationKind::Warning,
                    "Générez un CV avant l'export PDF.",
                );
                return Task::none();
            };
            let profile = &app.data.profile.personal;
            let mut lines = vec![generation.cv.summary.clone()];
            lines.extend(generation.cv.experiences.iter().flat_map(|experience| {
                [
                    format!("{} - {}", experience.title, experience.company),
                    experience.description.clone(),
                ]
            }));
            lines.push(format!("Compétences : {}", generation.cv.skills.join(", ")));
            let layout = crate::core::cv_pdf::CvLayout {
                name: format!("{} {}", profile.first_name, profile.last_name),
                headline: profile.headline.clone().unwrap_or_default(),
                lines,
            };
            return Task::perform(
                async move {
                    let Some(file) = rfd::AsyncFileDialog::new()
                        .set_title("Exporter le CV en PDF")
                        .set_file_name("cv-candilog.pdf")
                        .add_filter("PDF", &["pdf"])
                        .save_file()
                        .await
                    else {
                        return Err("Export annulé.".into());
                    };
                    let path = file.path().to_path_buf();
                    layout
                        .render_pdf(&path)
                        .map_err(|error| error.to_string())?;
                    Ok(path)
                },
                Message::CvPdfExported,
            );
        }
        Message::CvPdfExported(result) => match result {
            Ok(path) => app.notify_success(format!("CV exporté : {}", path.display())),
            Err(error) => app.notify_failure(error),
        },
        Message::SelectProfilePdf => {
            return Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Importer un CV dans le profil")
                        .add_filter("PDF", &["pdf"])
                        .pick_file()
                        .await
                        .map(|file| file.path().to_path_buf())
                },
                Message::ProfilePdfSelected,
            );
        }
        Message::ProfilePdfSelected(path) => app.profile_import_path = path,
        Message::ExtractProfile => {
            let (Some(backend), Some(path)) =
                (app.backend.clone(), app.profile_import_path.clone())
            else {
                app.notify(NotificationKind::Warning, "Sélectionnez un CV PDF.");
                return Task::none();
            };
            let token = tokio_util::sync::CancellationToken::new();
            let sequence = app.commencer_operation_ia(token.clone());
            app.extracted_profile = None;
            return Task::perform(
                async move {
                    crate::modules::ia::service::extract_profile_from_pdf(&backend, &path, token)
                        .await
                        .map_err(|error| error.to_string())
                },
                move |result| Message::ProfileExtracted(result, sequence),
            );
        }
        Message::ProfileExtracted(result, sequence) => {
            if !app.terminer_operation_ia(sequence) {
                return Task::none();
            }
            match result {
                Ok(profile) => {
                    app.extracted_profile = Some(profile);
                    app.notify_success(format!(
                        "Profil extrait en {} s. Vérifiez-le avant validation.",
                        app.ai_elapsed_seconds
                    ));
                }
                Err(error) => app.notify_failure(error),
            }
        }
        Message::ApplyExtractedProfile => {
            let Some(profile) = app.extracted_profile.clone() else {
                app.notify(
                    NotificationKind::Warning,
                    "Aucun profil extrait à appliquer.",
                );
                return Task::none();
            };
            let result = app.backend.as_ref().map_or_else(
                || Err("La base Candilog n'est pas disponible.".into()),
                |backend| {
                    backend
                        .profil
                        .update(&profile)
                        .map_err(|error| error.to_string())
                },
            );
            match result {
                Ok(_) => {
                    app.extracted_profile = None;
                    app.notify_success("Profil importé après validation.");
                    return recharger(app);
                }
                Err(error) => app.notify_failure(error),
            }
        }
        Message::AnalyzeInterview(id) => {
            let Some(backend) = app.backend.clone() else {
                app.notify_failure("La base Candilog n'est pas disponible.");
                return Task::none();
            };
            let token = tokio_util::sync::CancellationToken::new();
            let sequence = app.commencer_operation_ia(token.clone());
            return Task::perform(
                async move {
                    crate::modules::ia::service::analyze_interview(&backend, id, token)
                        .await
                        .map_err(|error| error.to_string())
                },
                move |result| Message::InterviewAnalyzed(result, sequence),
            );
        }
        Message::InterviewAnalyzed(result, sequence) => {
            if !app.terminer_operation_ia(sequence) {
                return Task::none();
            }
            match result {
                Ok(_) => {
                    app.notify_success("Compte rendu analysé et enregistré.");
                    return recharger(app);
                }
                Err(error) => app.notify_failure(error),
            }
        }
        Message::SelectBackupImport => {
            return Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Importer un backup Candilog")
                        .add_filter("SQLite", &["sqlite"])
                        .pick_file()
                        .await
                        .map(|file| file.path().to_path_buf())
                },
                Message::BackupImportSelected,
            );
        }
        Message::ProbeProviderHealth => {
            let config = app.data.settings.llm.clone();
            app.provider_health = crate::ui::components::runtime_status::Health::Checking;
            return Task::perform(
                async move {
                    let pin = crate::shared::llm::validate_llm_endpoint(&config)
                        .await
                        .map_err(|error| error.to_string())?;
                    crate::modules::ia::factory::build_provider_pinned(&config, pin)
                        .health_check()
                        .await
                        .map_err(|error| error.to_string())
                },
                Message::ProviderHealthChecked,
            );
        }
        Message::ProviderHealthChecked(result) => {
            // Contrôle silencieux : il alimente la pastille sans interrompre l'utilisateur.
            app.provider_health = match &result {
                Ok(()) => crate::ui::components::runtime_status::Health::Ok,
                Err(erreur) => {
                    tracing::warn!(erreur = %erreur, "fournisseur IA injoignable");
                    crate::ui::components::runtime_status::Health::Error
                }
            };
        }
        Message::RetryBootstrap => {
            app.bootstrap();
            if app.fatal_error.is_none() {
                app.notify_success("Base rouverte.");
            }
        }
        Message::SelectRecoveryBackup => {
            return Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Restaurer Candilog depuis un backup")
                        .add_filter("SQLite", &["sqlite"])
                        .pick_file()
                        .await
                        .map(|file| file.path().to_path_buf())
                },
                Message::RecoveryBackupSelected,
            );
        }
        Message::RecoveryBackupSelected(path) => {
            let Some(path) = path else {
                return Task::none();
            };
            // La base active étant inutilisable, la restauration passe par le système de
            // fichiers : aucun pool n'existe pour y appliquer l'API backup de SQLite.
            match crate::core::config::AppPaths::discover()
                .and_then(|paths| crate::core::backup::restore_file(&path, &paths.database))
            {
                Ok(()) => {
                    app.bootstrap();
                    if app.fatal_error.is_none() {
                        app.notify_success(
                            "Backup restauré. Candilog a redémarré sur vos données.",
                        );
                    }
                }
                Err(error) => {
                    tracing::error!(erreur = %error, "restauration de secours impossible");
                    app.fatal_error = Some(error.message_utilisateur());
                }
            }
        }
        Message::QuarantineDatabase => {
            match crate::core::config::AppPaths::discover()
                .and_then(|paths| crate::core::backup::quarantine(&paths.database))
            {
                Ok(ancienne) => {
                    app.bootstrap();
                    if app.fatal_error.is_none() {
                        app.notify(
                            NotificationKind::Warning,
                            format!(
                                "Candilog a redémarré sur une base neuve. L'ancienne base est \
                                 conservée sous « {} » au cas où elle serait récupérable.",
                                ancienne.file_name().unwrap_or_default().to_string_lossy()
                            ),
                        );
                    }
                }
                Err(error) => {
                    tracing::error!(erreur = %error, "mise de côté impossible");
                    app.fatal_error = Some(error.message_utilisateur());
                }
            }
        }
        Message::BackupImportSelected(path) => {
            if let Some(path) = path {
                match crate::core::backup::validate(&path) {
                    Ok(()) => {
                        app.pending_backup_import = Some(path);
                        app.dialog = Some(Dialog::ImportBackup);
                    }
                    Err(error) => app.notify_error(&error),
                }
            }
        }
        Message::ConfirmBackupImport => {
            let (Some(backend), Some(path)) =
                (app.backend.as_ref(), app.pending_backup_import.clone())
            else {
                app.notify(
                    NotificationKind::Warning,
                    "Aucun backup valide sélectionné.",
                );
                return Task::none();
            };
            match crate::core::backup::import(&backend.sqlite, &backend.db_path, &path) {
                Ok(()) => {
                    app.pending_backup_import = None;
                    app.dialog = None;
                    app.notify_success("Backup restauré avec succès.");
                    return recharger(app);
                }
                Err(error) => app.notify_error(&error),
            }
        }
        Message::ConfirmDatabaseReset => {
            let Some(backend) = app.backend.as_ref() else {
                app.notify_failure("La base Candilog n'est pas disponible.");
                return Task::none();
            };
            match crate::core::backup::reset_data(&backend.sqlite) {
                Ok(()) => {
                    app.dialog = None;
                    app.notify_success("Toutes les données ont été réinitialisées.");
                    return recharger(app);
                }
                Err(error) => app.notify_error(&error),
            }
        }
        Message::ConfirmAiCacheReset => {
            let Some(backend) = app.backend.as_ref() else {
                app.notify_failure("La base Candilog n'est pas disponible.");
                return Task::none();
            };
            match backend.cache_ia.reset() {
                Ok(()) => {
                    app.dialog = None;
                    app.notify_success("Cache IA vidé.");
                }
                Err(error) => app.notify_error(&error),
            }
        }
        Message::AcceptRecommendation(index) => {
            let recommendation = app
                .cv_generation
                .as_ref()
                .and_then(|generation| generation.analysis.recommandations.get(index))
                .cloned();
            if let (Some(generation), Some(recommendation)) =
                (app.cv_generation.as_mut(), recommendation)
            {
                apply_recommendation(&mut generation.cv, &recommendation);
                if let Some(status) = app.recommendation_states.get_mut(index) {
                    *status = super::state::RecommendationStatus::Accepted;
                }
            }
        }
        Message::RejectRecommendation(index) => {
            if let Some(status) = app.recommendation_states.get_mut(index) {
                *status = super::state::RecommendationStatus::Rejected;
            }
        }
        Message::SelectCandidate(id) => app.selected_candidate = id,
        Message::SelectCompany(id) => app.selected_company = id,
        Message::SelectContact(id) => app.selected_contact = id,
        Message::SelectCvVersion(id) => app.selected_cv = id,
        Message::ToggleFilters => app.filters_open = !app.filters_open,
        Message::SortCandidates(column) => {
            if let Some(sort) = super::state::CandidateSort::from_column(column) {
                if sort == app.candidate_sort {
                    app.candidate_sort_descending = !app.candidate_sort_descending;
                } else {
                    app.candidate_sort = sort;
                    app.candidate_sort_descending = false;
                }
            }
        }
        Message::StatisticsTabChanged(tab) => app.statistics_tab = tab,
        // Les compteurs sont 1-based et bornés **ici**, dans la transition d'état, et non
        // seulement à l'affichage : borner en aval laissait le compteur dériver sans que rien
        // ne bouge à l'écran, puis exiger autant de clics en sens inverse pour revenir dans
        // la plage utile.
        Message::AtsPagePrev => {
            app.ats_page = app.ats_page.saturating_sub(1).max(1);
            return recharger(app);
        }
        Message::AtsPageNext => {
            app.ats_page = app.ats_page.saturating_add(1).min(app.ats_total_pages());
            return recharger(app);
        }
        Message::LlmPagePrev => {
            app.llm_page = app.llm_page.saturating_sub(1).max(1);
            return recharger(app);
        }
        Message::LlmPageNext => {
            app.llm_page = app.llm_page.saturating_add(1).min(app.llm_total_pages());
            return recharger(app);
        }
        Message::DocumentWidthChanged(width) => app.document_width = width,
        Message::FocusSearch => {
            return iced::widget::text_input::focus(SEARCH_FIELD_ID);
        }
    }
    Task::none()
}

/// Identifiant du champ de recherche de l'écran courant, ciblé par `Ctrl+F`.
pub const SEARCH_FIELD_ID: &str = "candilog-search";

fn optional(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn apply_recommendation(
    cv: &mut crate::modules::ia::cv_model::GeneratedCv,
    recommendation: &crate::modules::ia::cv_model::RecommandationAts,
) {
    match recommendation.section.as_str() {
        "resume" | "summary" => cv.summary.clone_from(&recommendation.texte_propose),
        "competences" | "skills" => {
            cv.skills = recommendation
                .texte_propose
                .split([',', ';', '\n'])
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .collect();
        }
        section if section.starts_with("experience_") => {
            let index = section
                .trim_start_matches("experience_")
                .parse::<usize>()
                .ok();
            if let Some(experience) = index.and_then(|index| cv.experiences.get_mut(index)) {
                experience
                    .description
                    .clone_from(&recommendation.texte_propose);
            }
        }
        _ => {}
    }
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
