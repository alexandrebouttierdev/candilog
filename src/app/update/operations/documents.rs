//! Sous-domaine d'opérations applicatives.

use super::super::*;

pub(super) fn handles(message: &Message) -> bool {
    matches!(
        message,
        Message::CvVersionNameChanged(..)
            | Message::SaveGeneratedCv
            | Message::LoadCvVersion(..)
            | Message::ExportGeneratedCvPdf
            | Message::CvPdfExported(..)
            | Message::SelectProfilePdf
            | Message::ProfilePdfSelected(..)
            | Message::ExtractProfile
            | Message::ProfileExtracted(..)
            | Message::ToggleProfileImportItem(..)
            | Message::AcceptAllProfileImportItems
            | Message::RejectAllProfileImportItems
            | Message::ApplyExtractedProfile
    )
}

pub(super) fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
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
                                            crate::app::state::RecommendationStatus::Pending;
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
            let document =
                crate::modules::ia::cv_document::construire(&app.data.profile, generation);
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
                    document
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
        Message::ProfilePdfSelected(path) => {
            if path.is_some() {
                app.extracted_profile = None;
                app.profile_import_excluded.clear();
                if app.dialog == Some(Dialog::ProfileImport) {
                    app.dialog = None;
                }
            }
            app.profile_import_path = path;
        }
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
                    app.profile_import_excluded.clear();
                    app.dialog = Some(Dialog::ProfileImport);
                    app.notify_success(format!(
                        "Profil extrait en {}. Vérifiez-le avant validation.",
                        ui_format::duree(app.ai_elapsed_seconds)
                    ));
                    return sonner_fin_analyse();
                }
                Err(error) => app.notify_failure(error),
            }
        }
        Message::ToggleProfileImportItem(key) => {
            if !app.profile_import_excluded.remove(&key) {
                app.profile_import_excluded.insert(key);
            }
        }
        Message::AcceptAllProfileImportItems => app.profile_import_excluded.clear(),
        Message::RejectAllProfileImportItems => {
            app.profile_import_excluded = app
                .extracted_profile
                .as_ref()
                .map_or_else(std::collections::HashSet::new, all_import_item_keys);
        }
        Message::ApplyExtractedProfile => {
            let Some(profile) = app.extracted_profile.clone() else {
                app.notify(
                    NotificationKind::Warning,
                    "Aucun profil extrait à appliquer.",
                );
                return Task::none();
            };
            let filtered = filter_imported_profile(&profile, &app.profile_import_excluded);
            let merged = merge_imported_profile(&app.data.profile, &filtered);
            let result = app.backend.as_ref().map_or_else(
                || Err("La base Candilog n'est pas disponible.".into()),
                |backend| {
                    backend
                        .profil
                        .update(&merged)
                        .map_err(|error| error.to_string())
                },
            );
            match result {
                Ok(_) => {
                    app.extracted_profile = None;
                    app.profile_import_excluded.clear();
                    app.dialog = None;
                    app.notify_success("Profil importé après validation.");
                    return recharger(app);
                }
                Err(error) => app.notify_failure(error),
            }
        }
        _ => unreachable!("message routé vers un sous-domaine incorrect"),
    }
    Task::none()
}
