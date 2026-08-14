//! Transitions d'état du domaine `operations`.

use super::*;

mod candidates;
mod documents;

pub(super) fn handles(message: &Message) -> bool {
    matches!(
        message,
        Message::UpdateDownload(..)
            | Message::CandidateFilterStatus(..)
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
            | Message::CvVersionNameChanged(..)
            | Message::SaveGeneratedCv
            | Message::LoadCvVersion(..)
            | Message::PreviewCvVersion(..)
            | Message::ExportGeneratedCvPdf
            | Message::CvPdfExported(..)
            | Message::SaveLetter
            | Message::LoadLetter(..)
            | Message::SelectProfilePdf
            | Message::ProfilePdfSelected(..)
            | Message::ExtractProfile
            | Message::ProfileExtracted(..)
            | Message::ToggleProfileImportItem(..)
            | Message::AcceptAllProfileImportItems
            | Message::RejectAllProfileImportItems
            | Message::ApplyExtractedProfile
            | Message::AnalyzeInterview(..)
            | Message::InterviewAnalyzed(..)
            | Message::SelectBackupImport
            | Message::ProbeProviderHealth
            | Message::ProviderHealthChecked(..)
            | Message::RetryBootstrap
            | Message::SelectRecoveryBackup
            | Message::RecoveryBackupSelected(..)
            | Message::QuarantineDatabase
            | Message::BackupImportSelected(..)
            | Message::ConfirmBackupImport
            | Message::ConfirmDatabaseReset
            | Message::ConfirmAiCacheReset
    )
}

pub(super) fn update(app: &mut App, message: Message) -> Task<Message> {
    if candidates::handles(&message) {
        return candidates::update(app, message);
    }
    if documents::handles(&message) {
        return documents::update(app, message);
    }
    match message {
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
                    return sonner_fin_analyse().chain(recharger(app));
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
            let Some(backend) = app.backend.clone() else {
                app.provider_health = crate::ui::components::runtime_status::Health::Error;
                return Task::none();
            };
            app.provider_health = crate::ui::components::runtime_status::Health::Checking;
            return Task::perform(
                async move {
                    let settings = backend
                        .secure_settings_async()
                        .await
                        .map_err(|error| error.to_string())?;
                    let config = settings.llm;
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
            let Some(path) = app.pending_backup_import.clone() else {
                app.notify(
                    NotificationKind::Warning,
                    "Aucun backup valide sélectionné.",
                );
                return Task::none();
            };
            return ecrire(app, "Backup restauré avec succès.", move |backend| {
                crate::core::backup::import(&backend.sqlite, &backend.db_path, &path)
                    .map_err(|error| error.to_string())
            });
        }
        Message::ConfirmDatabaseReset => {
            return ecrire(
                app,
                "Toutes les données ont été réinitialisées.",
                |backend| {
                    crate::core::backup::reset_data(&backend.sqlite)
                        .map_err(|error| error.to_string())
                },
            );
        }
        Message::ConfirmAiCacheReset => {
            return ecrire(app, "Cache IA vidé.", |backend| {
                backend.cache_ia.reset().map_err(|error| error.to_string())
            });
        }
        _ => unreachable!("message routé vers un domaine incorrect"),
    }
    Task::none()
}
