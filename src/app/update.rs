//! Traitement des messages Iced.

use super::message::{LetterStreamEvent, UpdateDownloadEvent};
use super::state::{
    CandidatureForm, ContactForm, Dialog, EntrepriseForm, EntretienForm, RelanceForm,
};
use super::{App, Message};
use crate::modules::candidatures::model::NouvelleCandidature;
use crate::modules::contacts::model::NouveauContact;
use crate::modules::entreprises::model::NouvelleEntreprise;
use crate::modules::entretiens::model::NouvelEntretien;
use crate::modules::ia::cache::CacheIaRepository;
use crate::modules::relances::model::NouvelleRelance;
use chrono::{Datelike, Local};
use iced::futures::SinkExt;
use iced::{keyboard, Subscription, Task, Theme};
use semver::Version;
use std::time::Duration;

/// Met à jour l'état applicatif.
pub fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::CaptureForReview => {
            return iced::window::get_latest().map(Message::CaptureWindow);
        }
        Message::CaptureWindow(Some(id)) => {
            return iced::window::screenshot(id).map(Message::CapturedForReview);
        }
        Message::CaptureWindow(None) => {
            app.notification = Some("Capture visuelle : fenêtre introuvable.".into());
        }
        Message::CapturedForReview(screenshot) => {
            if let Err(error) = save_review_screenshot(&screenshot) {
                app.notification = Some(error);
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
            app.route = route;
            app.search.clear();
        }
        Message::Reload => app.reload(),
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
        Message::ToggleTheme => app.is_dark = !app.is_dark,
        Message::Tick => {
            if app.ai_is_running {
                app.ai_elapsed_seconds = app.ai_elapsed_seconds.saturating_add(1);
            }
        }
        Message::CheckUpdate => {
            let client = reqwest::Client::new();
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
                app.notification = Some(format!("Candilog {} est disponible.", info.version));
                app.available_update = Some(info);
            }
            Ok(None) => app.notification = Some("Candilog est à jour.".into()),
            Err(error) => app.notification = Some(format!("Vérification impossible : {error}")),
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
            app.selected_contact = None;
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
            let result = app.backend.as_ref().map_or_else(
                || Err("La base Candilog n'est pas disponible.".into()),
                |backend| {
                    app.editing_id
                        .map_or_else(
                            || backend.entreprises.creer(&input),
                            |id| backend.entreprises.modifier(id, &input),
                        )
                        .map_err(|error| error.to_string())
                },
            );
            finish_submit(app, result.map(|_| ()), "Entreprise enregistrée.");
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
            let result = app.backend.as_ref().map_or_else(
                || Err("La base Candilog n'est pas disponible.".into()),
                |backend| {
                    app.editing_id
                        .map_or_else(
                            || backend.contacts.creer(&input),
                            |id| backend.contacts.modifier(id, &input),
                        )
                        .map_err(|error| error.to_string())
                },
            );
            finish_submit(app, result.map(|_| ()), "Contact enregistré.");
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
            let result = app.candidature_form.entreprise_id.map_or_else(
                || Err("Sélectionnez une entreprise.".into()),
                |entreprise_id| {
                    let input = NouvelleCandidature {
                        poste: app.candidature_form.poste.clone(),
                        entreprise_id,
                        type_contrat: app.candidature_form.type_contrat,
                        statut: app.candidature_form.statut,
                        date_envoi: app.candidature_form.date_envoi.clone(),
                        lien_offre: optional(&app.candidature_form.lien_offre),
                        notes: optional(&app.candidature_form.notes),
                    };
                    app.backend.as_ref().map_or_else(
                        || Err("La base Candilog n'est pas disponible.".into()),
                        |backend| {
                            app.editing_id
                                .map_or_else(
                                    || backend.candidatures.creer(&input),
                                    |id| backend.candidatures.modifier(id, &input),
                                )
                                .map_err(|error| error.to_string())
                        },
                    )
                },
            );
            finish_submit(app, result.map(|_| ()), "Candidature enregistrée.");
        }
        Message::MoveCandidature(id, status) => {
            let result = app.backend.as_ref().map_or_else(
                || Err("La base Candilog n'est pas disponible.".into()),
                |backend| {
                    backend
                        .candidatures
                        .changer_statut(id, status)
                        .map_err(|error| error.to_string())
                },
            );
            finish_submit(app, result.map(|_| ()), "Statut mis à jour.");
        }
        Message::EntretienCandidatureChanged(value) => {
            app.entretien_form.candidature_id = Some(value)
        }
        Message::EntretienContactChanged(value) => app.entretien_form.contact_id = value,
        Message::EntretienDateChanged(value) => app.entretien_form.date_entretien = value,
        Message::EntretienTypeChanged(value) => app.entretien_form.type_entretien = value,
        Message::EntretienLieuChanged(value) => app.entretien_form.lieu = value,
        Message::EntretienNotesChanged(value) => app.entretien_form.notes = value,
        Message::EntretienCompteRenduChanged(value) => app.entretien_form.compte_rendu = value,
        Message::SubmitEntretien => {
            let result = app.entretien_form.candidature_id.map_or_else(
                || Err("Sélectionnez une candidature.".into()),
                |candidature_id| {
                    let input = NouvelEntretien {
                        candidature_id,
                        contact_id: app.entretien_form.contact_id,
                        date_entretien: app.entretien_form.date_entretien.clone(),
                        type_entretien: app.entretien_form.type_entretien,
                        lieu: optional(&app.entretien_form.lieu),
                        notes: optional(&app.entretien_form.notes),
                        compte_rendu: optional(&app.entretien_form.compte_rendu),
                    };
                    app.backend.as_ref().map_or_else(
                        || Err("La base Candilog n'est pas disponible.".into()),
                        |backend| {
                            app.editing_id
                                .map_or_else(
                                    || backend.entretiens.creer(&input),
                                    |id| backend.entretiens.modifier(id, &input),
                                )
                                .map_err(|error| error.to_string())
                        },
                    )
                },
            );
            finish_submit(app, result.map(|_| ()), "Entretien enregistré.");
        }
        Message::RelanceCandidatureChanged(value) => app.relance_form.candidature_id = Some(value),
        Message::RelanceDateChanged(value) => app.relance_form.date_relance = value,
        Message::RelanceTypeChanged(value) => app.relance_form.type_relance = value,
        Message::RelanceNotesChanged(value) => app.relance_form.notes = value,
        Message::SubmitRelance => {
            let result = app.relance_form.candidature_id.map_or_else(
                || Err("Sélectionnez une candidature.".into()),
                |candidature_id| {
                    let input = NouvelleRelance {
                        candidature_id,
                        date_relance: app.relance_form.date_relance.clone(),
                        type_relance: app.relance_form.type_relance.clone(),
                        notes: optional(&app.relance_form.notes),
                    };
                    app.backend.as_ref().map_or_else(
                        || Err("La base Candilog n'est pas disponible.".into()),
                        |backend| {
                            app.editing_id
                                .map_or_else(
                                    || backend.relances.creer(&input),
                                    |id| backend.relances.modifier(id, &input),
                                )
                                .map_err(|error| error.to_string())
                        },
                    )
                },
            );
            finish_submit(app, result.map(|_| ()), "Relance enregistrée.");
        }
        Message::ExportCandidatures => {
            let rows = app.filtered_candidates().into_iter().cloned().collect();
            return Task::perform(export_candidatures(rows), Message::CandidaturesExported);
        }
        Message::CandidaturesExported(result) => match result {
            Ok(path) => app.notification = Some(format!("Export créé : {}", path.display())),
            Err(error) => app.notification = Some(error),
        },
        Message::OfferEditorAction(action) => app.offer_editor.perform(action),
        Message::AnalyzeOffer => {
            let Some(backend) = app.backend.clone() else {
                app.notification = Some("La base Candilog n'est pas disponible.".into());
                return Task::none();
            };
            let offer = app.offer_editor.text();
            app.ai_is_running = true;
            app.ai_elapsed_seconds = 0;
            app.offer_analysis = None;
            app.cv_generation = None;
            let token = tokio_util::sync::CancellationToken::new();
            app.ai_cancellation = Some(token.clone());
            return Task::perform(
                async move {
                    tokio::select! {
                        result = crate::modules::ia::service::analyze_offer(&backend, offer) => {
                            result.map_err(|error| error.to_string())
                        }
                        () = token.cancelled() => Err("Génération annulée".into()),
                    }
                },
                Message::OfferAnalyzed,
            );
        }
        Message::OfferAnalyzed(result) => {
            app.ai_is_running = false;
            app.ai_cancellation = None;
            match result {
                Ok(analysis) => {
                    app.offer_analysis = Some(analysis);
                    app.notification =
                        Some(format!("Offre analysée en {} s.", app.ai_elapsed_seconds));
                }
                Err(error) => app.notification = Some(error),
            }
        }
        Message::GenerateCv => {
            let (Some(backend), Some(analysis)) = (app.backend.clone(), app.offer_analysis.clone())
            else {
                app.notification = Some("Analysez d'abord une offre.".into());
                return Task::none();
            };
            let token = tokio_util::sync::CancellationToken::new();
            app.ai_cancellation = Some(token.clone());
            app.ai_is_running = true;
            app.ai_elapsed_seconds = 0;
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
                Message::CvGenerated,
            );
        }
        Message::CvGenerated(result) => {
            app.ai_is_running = false;
            app.ai_cancellation = None;
            match result {
                Ok(generation) => {
                    app.recommendation_states = vec![
                        super::state::RecommendationStatus::Pending;
                        generation.analysis.recommandations.len()
                    ];
                    app.cv_generation = Some(generation);
                    app.notification = Some(format!("CV généré en {} s.", app.ai_elapsed_seconds));
                }
                Err(error) => app.notification = Some(error),
            }
        }
        Message::CancelAi => {
            if let Some(token) = &app.ai_cancellation {
                token.cancel();
                app.notification = Some("Annulation demandée…".into());
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
                app.notification = Some("Sélectionnez un PDF avant l'analyse.".into());
                return Task::none();
            };
            let offer = app.import_offer_editor.text();
            let token = tokio_util::sync::CancellationToken::new();
            app.ai_cancellation = Some(token.clone());
            app.ai_is_running = true;
            app.ai_elapsed_seconds = 0;
            app.imported_cv_analysis = None;
            return Task::perform(
                async move {
                    crate::modules::ia::service::analyze_imported_cv(&backend, &path, offer, token)
                        .await
                        .map_err(|error| error.to_string())
                },
                Message::ImportedCvAnalyzed,
            );
        }
        Message::ImportedCvAnalyzed(result) => {
            app.ai_is_running = false;
            app.ai_cancellation = None;
            match result {
                Ok(analysis) => {
                    app.imported_cv_analysis = Some(analysis);
                    app.notification = Some(format!("CV analysé en {} s.", app.ai_elapsed_seconds));
                }
                Err(error) => app.notification = Some(error),
            }
        }
        Message::LetterCompanyChanged(value) => app.letter_company = value,
        Message::LetterJobTitleChanged(value) => app.letter_job_title = value,
        Message::LetterEditorAction(action) => app.letter_editor.perform(action),
        Message::LetterToneChanged(value) => app.letter_tone = value,
        Message::LetterLengthChanged(value) => app.letter_length = value,
        Message::GenerateLetter => {
            let Some(backend) = app.backend.clone() else {
                app.notification = Some("La base Candilog n'est pas disponible.".into());
                return Task::none();
            };
            let token = tokio_util::sync::CancellationToken::new();
            app.ai_cancellation = Some(token.clone());
            app.ai_is_running = true;
            app.ai_elapsed_seconds = 0;
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
            return Task::run(stream, Message::LetterStream);
        }
        Message::LetterStream(event) => match event {
            LetterStreamEvent::Chunk(chunk) => app.letter_output.push_str(&chunk),
            LetterStreamEvent::Finished(result) => {
                app.ai_is_running = false;
                app.ai_cancellation = None;
                match result {
                    Ok(letter) => {
                        app.letter_output = letter;
                        app.notification =
                            Some(format!("Lettre générée en {} s.", app.ai_elapsed_seconds));
                    }
                    Err(error) => app.notification = Some(error),
                }
            }
        },
        Message::SettingsProviderChanged(provider) => {
            app.data.settings.llm.provider = provider;
            if matches!(
                app.data.settings.llm.provider,
                crate::shared::llm::ProviderKind::Ollama
            ) && app.data.settings.llm.endpoint.is_none()
            {
                app.data.settings.llm.endpoint = Some("http://localhost:11434".into());
            }
        }
        Message::SettingsModelChanged(value) => app.data.settings.llm.model = value,
        Message::SettingsEndpointChanged(value) => {
            app.data.settings.llm.endpoint = optional(&value)
        }
        Message::SettingsApiKeyChanged(value) => app.data.settings.llm.api_key = optional(&value),
        Message::SettingsTemperatureChanged(value) => app.data.settings.llm.temperature = value,
        Message::SettingsModeChanged(value) => app.data.settings.llm.mode = value,
        Message::SettingsThemeChanged(value) => {
            app.is_dark = !matches!(value, crate::modules::settings::model::ThemePref::Light);
            app.data.settings.theme = value;
        }
        Message::SaveSettings => {
            let Some(backend) = app.backend.clone() else {
                app.notification = Some("La base Candilog n'est pas disponible.".into());
                return Task::none();
            };
            let settings = app.data.settings.clone();
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
                app.data.settings = settings;
                app.notification = Some("Paramètres enregistrés.".into());
            }
            Err(error) => app.notification = Some(error),
        },
        Message::TestLlmConnection => {
            let config = app.data.settings.llm.clone();
            return Task::perform(
                async move {
                    crate::shared::llm::validate_llm_endpoint(&config)
                        .await
                        .map_err(|error| error.to_string())?;
                    crate::modules::ia::factory::build_provider(&config)
                        .health_check()
                        .await
                        .map_err(|error| error.to_string())
                },
                Message::LlmConnectionTested,
            );
        }
        Message::LlmConnectionTested(result) => match result {
            Ok(()) => app.notification = Some("Connexion IA opérationnelle.".into()),
            Err(error) => app.notification = Some(format!("Connexion IA impossible : {error}")),
        },
        Message::ExportBackup => {
            let Some(backend) = app.backend.clone() else {
                app.notification = Some("La base Candilog n'est pas disponible.".into());
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
            Ok(path) => app.notification = Some(format!("Backup créé : {}", path.display())),
            Err(error) => app.notification = Some(error),
        },
        Message::CandidateDragStarted(id) => {
            app.dragging_candidate = Some(id);
            app.drag_target_status = None;
        }
        Message::CandidateDragHovered(status) => app.drag_target_status = Some(status),
        Message::CandidateDropped(status) => {
            let Some(id) = app.dragging_candidate.take() else {
                return Task::none();
            };
            app.drag_target_status = None;
            let result = app.backend.as_ref().map_or_else(
                || Err("La base Candilog n'est pas disponible.".into()),
                |backend| {
                    backend
                        .candidatures
                        .changer_statut(id, status)
                        .map_err(|error| error.to_string())
                },
            );
            finish_submit(
                app,
                result.map(|_| ()),
                "Statut mis à jour par glisser-déposer.",
            );
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
            let result = app.backend.as_ref().map_or_else(
                || Err("La base Candilog n'est pas disponible.".into()),
                |backend| {
                    backend
                        .profil
                        .update(&profile)
                        .map_err(|error| error.to_string())
                },
            );
            finish_submit(app, result.map(|_| ()), "Profil enregistré.");
        }
        Message::DownloadUpdate => {
            let Some(update) = app.available_update.clone() else {
                app.notification = Some("Aucune mise à jour disponible.".into());
                return Task::none();
            };
            app.update_progress = Some(0);
            app.verified_update_path = None;
            let stream = iced::stream::channel(32, move |mut sender| async move {
                let client = reqwest::Client::new();
                let mut progress_sender = sender.clone();
                let result = crate::core::updater::download_verified(&client, &update, |value| {
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
                    app.notification = Some(format!(
                        "Mise à jour téléchargée et signature vérifiée : {}",
                        path.display()
                    ));
                }
                Err(error) => {
                    app.update_progress = None;
                    app.notification = Some(error);
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
            let result = app.backend.as_ref().map_or_else(
                || Err("La base Candilog n'est pas disponible.".into()),
                |backend| {
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
                },
            );
            finish_submit(app, result, "Élément supprimé.");
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
                    date_envoi: item.date_envoi.clone(),
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
                    date_entretien: item.date_entretien.clone(),
                    type_entretien: item.type_entretien,
                    lieu: item.lieu.clone().unwrap_or_default(),
                    notes: item.notes.clone().unwrap_or_default(),
                    compte_rendu: item.compte_rendu.clone().unwrap_or_default(),
                };
                app.editing_id = Some(id);
                app.dialog = Some(Dialog::Entretien);
            }
        }
        Message::EditRelance(id) => {
            if let Some(item) = app.data.relances.iter().find(|item| item.id == id) {
                app.relance_form = RelanceForm {
                    candidature_id: Some(item.candidature_id),
                    date_relance: item.date_relance.clone(),
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
                app.notification = Some("Générez un CV avant de le sauvegarder.".into());
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
                    app.reload();
                    app.notification = Some("Version de CV sauvegardée.".into());
                }
                Err(error) => app.notification = Some(error.to_string()),
            }
        }
        Message::LoadCvVersion(id) => {
            let Some(backend) = app.backend.as_ref() else {
                app.notification = Some("La base Candilog n'est pas disponible.".into());
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
                        _ => app.notification = Some("Cette version de CV est illisible.".into()),
                    }
                }
                Err(error) => app.notification = Some(error.to_string()),
            }
        }
        Message::ExportGeneratedCvPdf => {
            let Some(generation) = app.cv_generation.as_ref() else {
                app.notification = Some("Générez un CV avant l'export PDF.".into());
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
            Ok(path) => app.notification = Some(format!("CV exporté : {}", path.display())),
            Err(error) => app.notification = Some(error),
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
                app.notification = Some("Sélectionnez un CV PDF.".into());
                return Task::none();
            };
            let token = tokio_util::sync::CancellationToken::new();
            app.ai_cancellation = Some(token.clone());
            app.ai_is_running = true;
            app.ai_elapsed_seconds = 0;
            app.extracted_profile = None;
            return Task::perform(
                async move {
                    crate::modules::ia::service::extract_profile_from_pdf(&backend, &path, token)
                        .await
                        .map_err(|error| error.to_string())
                },
                Message::ProfileExtracted,
            );
        }
        Message::ProfileExtracted(result) => {
            app.ai_is_running = false;
            app.ai_cancellation = None;
            match result {
                Ok(profile) => {
                    app.extracted_profile = Some(profile);
                    app.notification = Some(format!(
                        "Profil extrait en {} s. Vérifiez-le avant validation.",
                        app.ai_elapsed_seconds
                    ));
                }
                Err(error) => app.notification = Some(error),
            }
        }
        Message::ApplyExtractedProfile => {
            let Some(profile) = app.extracted_profile.clone() else {
                app.notification = Some("Aucun profil extrait à appliquer.".into());
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
                    app.reload();
                    app.notification = Some("Profil importé après validation.".into());
                }
                Err(error) => app.notification = Some(error),
            }
        }
        Message::AnalyzeInterview(id) => {
            let Some(backend) = app.backend.clone() else {
                app.notification = Some("La base Candilog n'est pas disponible.".into());
                return Task::none();
            };
            let token = tokio_util::sync::CancellationToken::new();
            app.ai_cancellation = Some(token.clone());
            app.ai_is_running = true;
            app.ai_elapsed_seconds = 0;
            return Task::perform(
                async move {
                    crate::modules::ia::service::analyze_interview(&backend, id, token)
                        .await
                        .map_err(|error| error.to_string())
                },
                Message::InterviewAnalyzed,
            );
        }
        Message::InterviewAnalyzed(result) => {
            app.ai_is_running = false;
            app.ai_cancellation = None;
            match result {
                Ok(_) => {
                    app.reload();
                    app.notification = Some("Compte rendu analysé et enregistré.".into());
                }
                Err(error) => app.notification = Some(error),
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
        Message::BackupImportSelected(path) => {
            if let Some(path) = path {
                match crate::core::backup::validate(&path) {
                    Ok(()) => {
                        app.pending_backup_import = Some(path);
                        app.dialog = Some(Dialog::ImportBackup);
                    }
                    Err(error) => app.notification = Some(error.to_string()),
                }
            }
        }
        Message::ConfirmBackupImport => {
            let (Some(backend), Some(path)) =
                (app.backend.as_ref(), app.pending_backup_import.clone())
            else {
                app.notification = Some("Aucun backup valide sélectionné.".into());
                return Task::none();
            };
            match crate::core::backup::import(&backend.sqlite, &path) {
                Ok(()) => {
                    app.pending_backup_import = None;
                    app.dialog = None;
                    app.reload();
                    app.notification = Some("Backup restauré avec succès.".into());
                }
                Err(error) => app.notification = Some(error.to_string()),
            }
        }
        Message::ConfirmDatabaseReset => {
            let Some(backend) = app.backend.as_ref() else {
                app.notification = Some("La base Candilog n'est pas disponible.".into());
                return Task::none();
            };
            match crate::core::backup::reset_data(&backend.sqlite) {
                Ok(()) => {
                    app.dialog = None;
                    app.reload();
                    app.notification = Some("Toutes les données ont été réinitialisées.".into());
                }
                Err(error) => app.notification = Some(error.to_string()),
            }
        }
        Message::ConfirmAiCacheReset => {
            let Some(backend) = app.backend.as_ref() else {
                app.notification = Some("La base Candilog n'est pas disponible.".into());
                return Task::none();
            };
            match backend.cache_ia.reset() {
                Ok(()) => {
                    app.dialog = None;
                    app.notification = Some("Cache IA vidé.".into());
                }
                Err(error) => app.notification = Some(error.to_string()),
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
        Message::SettingsSectionChanged(section) => app.settings_section = section,
        Message::StatisticsTabChanged(tab) => app.statistics_tab = tab,
        Message::AtsPagePrev => app.ats_page = app.ats_page.saturating_sub(1),
        Message::AtsPageNext => app.ats_page = app.ats_page.saturating_add(1),
        Message::LlmPagePrev => app.llm_page = app.llm_page.saturating_sub(1),
        Message::LlmPageNext => app.llm_page = app.llm_page.saturating_add(1),
        Message::DocumentWidthChanged(width) => app.document_width = width,
        Message::FocusSearch => {
            return iced::widget::text_input::focus(SEARCH_FIELD_ID);
        }
    }
    Task::none()
}

/// Identifiant du champ de recherche de l'écran courant, ciblé par `Ctrl+F`.
pub const SEARCH_FIELD_ID: &str = "candilog-search";

fn save_review_screenshot(screenshot: &iced::window::Screenshot) -> Result<(), String> {
    let path = std::env::var_os("CANDILOG_CAPTURE_PATH")
        .ok_or_else(|| "Chemin de capture visuelle absent.".to_string())?;
    let pixel_count = u64::from(screenshot.size.width) * u64::from(screenshot.size.height);
    let expected = usize::try_from(pixel_count.saturating_mul(4))
        .map_err(|_| "Capture visuelle trop volumineuse.".to_string())?;
    if screenshot.bytes.len() != expected {
        return Err("Capture visuelle Iced incomplète.".into());
    }
    let mut ppm = format!(
        "P6\n{} {}\n255\n",
        screenshot.size.width, screenshot.size.height
    )
    .into_bytes();
    ppm.reserve(usize::try_from(pixel_count.saturating_mul(3)).unwrap_or_default());
    for rgba in screenshot.bytes.chunks_exact(4) {
        ppm.extend_from_slice(&rgba[..3]);
    }
    std::fs::write(path, ppm)
        .map_err(|error| format!("Impossible d'enregistrer la capture visuelle : {error}"))
}

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

fn finish_submit(app: &mut App, result: Result<(), String>, success: &str) {
    match result {
        Ok(()) => {
            app.dialog = None;
            app.editing_id = None;
            app.reload();
            app.notification = Some(success.into());
            let _ = notify_rust::Notification::new()
                .summary("Candilog")
                .body(success)
                .show();
        }
        Err(error) => app.notification = Some(error),
    }
}

async fn export_candidatures(
    rows: Vec<crate::modules::candidatures::model::Candidature>,
) -> Result<std::path::PathBuf, String> {
    let Some(file) = rfd::AsyncFileDialog::new()
        .set_title("Exporter les candidatures")
        .set_file_name("candidatures.csv")
        .add_filter("CSV", &["csv"])
        .save_file()
        .await
    else {
        return Err("Export annulé.".into());
    };
    let path = file.path().to_path_buf();
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_writer(Vec::new());
    writer
        .write_record([
            "poste",
            "entreprise",
            "contrat",
            "statut",
            "date_envoi",
            "lien_offre",
            "notes",
        ])
        .map_err(|error| format!("Impossible de préparer le CSV : {error}"))?;
    for row in rows {
        writer
            .write_record([
                row.poste,
                row.entreprise_nom.unwrap_or_default(),
                row.type_contrat.to_string(),
                row.statut.to_string(),
                row.date_envoi,
                row.lien_offre.unwrap_or_default(),
                row.notes.unwrap_or_default(),
            ])
            .map_err(|error| format!("Impossible d'écrire le CSV : {error}"))?;
    }
    let bytes = writer
        .into_inner()
        .map_err(|error| format!("Impossible de terminer le CSV : {error}"))?;
    std::fs::write(&path, bytes)
        .map_err(|error| format!("Impossible d'enregistrer le CSV : {error}"))?;
    Ok(path)
}

/// Abonnements desktop : chronomètre et fermeture clavier des surfaces temporaires.
pub fn subscription(_app: &App) -> Subscription<Message> {
    Subscription::batch([
        iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick),
        keyboard::on_key_press(shortcut),
        iced::window::resize_events().map(|(_id, size)| Message::WindowResized(size)),
    ])
}

/// Traduit une combinaison clavier en message applicatif.
fn shortcut(key: keyboard::Key, modifiers: keyboard::Modifiers) -> Option<Message> {
    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) {
        return Some(Message::CloseDialog);
    }
    if !modifiers.command() {
        return None;
    }
    let keyboard::Key::Character(character) = &key else {
        return None;
    };
    match character.as_str() {
        "n" => Some(Message::OpenDialog(super::state::Dialog::Candidature)),
        "f" => Some(Message::FocusSearch),
        "r" => Some(Message::Reload),
        digit => digit
            .chars()
            .next()
            .filter(char::is_ascii_digit)
            .and_then(crate::navigation::Route::from_shortcut)
            .map(Message::Navigate),
    }
}

/// Thème Iced actif.
pub fn theme(app: &App) -> Theme {
    if app.is_dark {
        crate::ui::theme::dark()
    } else {
        crate::ui::theme::light()
    }
}

#[cfg(test)]
#[path = "tests/update/mod.rs"]
mod tests;
