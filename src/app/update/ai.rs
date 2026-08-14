//! Transitions d'état du domaine `ai`.

use super::*;

pub(super) fn handles(message: &Message) -> bool {
    matches!(
        message,
        Message::OfferEditorAction(..)
            | Message::PasteOfferFromClipboard
            | Message::OfferClipboardRead(..)
            | Message::AnalyzeOffer
            | Message::OfferAnalyzed(..)
            | Message::GenerateCv
            | Message::CvGenerated(..)
            | Message::CancelAi
            | Message::SelectImportPdf
            | Message::ImportPdfSelected(..)
            | Message::ImportOfferEditorAction(..)
            | Message::AnalyzeImportedCv
            | Message::ImportedCvAnalyzed(..)
            | Message::LetterCompanyChanged(..)
            | Message::LetterJobTitleChanged(..)
            | Message::LetterEditorAction(..)
            | Message::PasteLetterFromClipboard
            | Message::LetterClipboardRead(..)
            | Message::LetterToneChanged(..)
            | Message::LetterLengthChanged(..)
            | Message::GenerateLetter
            | Message::LetterStream(..)
    )
}

pub(super) fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::OfferEditorAction(action) => app.offer_editor.perform(action),
        Message::PasteOfferFromClipboard => {
            return iced::clipboard::read().map(Message::OfferClipboardRead);
        }
        Message::OfferClipboardRead(value) => match value {
            Some(value) if !value.trim().is_empty() => {
                app.offer_editor = iced::widget::text_editor::Content::with_text(&value);
                app.offer_analysis = None;
                app.cv_generation = None;
                app.notify_success("Offre collée depuis le presse-papiers.");
            }
            Some(_) => app.notify(NotificationKind::Warning, "Le presse-papiers est vide."),
            None => app.notify_failure("Impossible de lire le presse-papiers."),
        },
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
                    app.notify_success(format!(
                        "Offre analysée en {}.",
                        ui_format::duree(app.ai_elapsed_seconds)
                    ));
                    return sonner_fin_analyse();
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
                        super::super::state::RecommendationStatus::Pending;
                        generation.analysis.recommandations.len()
                    ];
                    app.cv_generation = Some(generation);
                    let corps =
                        format!("CV généré en {}.", ui_format::duree(app.ai_elapsed_seconds));
                    app.notify_success(corps.clone());
                    return sonner_fin_analyse().chain(notifier_le_bureau(corps));
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
                    let corps = format!(
                        "CV analysé en {}.",
                        ui_format::duree(app.ai_elapsed_seconds)
                    );
                    app.notify_success(corps.clone());
                    return sonner_fin_analyse().chain(notifier_le_bureau(corps));
                }
                Err(error) => app.notify_failure(error),
            }
        }
        Message::LetterCompanyChanged(value) => app.letter_company = value,
        Message::LetterJobTitleChanged(value) => app.letter_job_title = value,
        Message::LetterEditorAction(action) => app.letter_editor.perform(action),
        Message::PasteLetterFromClipboard => {
            return iced::clipboard::read().map(Message::LetterClipboardRead);
        }
        Message::LetterClipboardRead(value) => match value {
            Some(value) if !value.trim().is_empty() => {
                app.letter_editor = iced::widget::text_editor::Content::with_text(&value);
                app.notify_success("Contexte collé depuis le presse-papiers.");
            }
            Some(_) => app.notify(NotificationKind::Warning, "Le presse-papiers est vide."),
            None => app.notify_failure("Impossible de lire le presse-papiers."),
        },
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
                        let corps = format!(
                            "Lettre générée en {}.",
                            ui_format::duree(app.ai_elapsed_seconds)
                        );
                        app.notify_success(corps.clone());
                        return sonner_fin_analyse().chain(notifier_le_bureau(corps));
                    }
                    Err(error) => app.notify_failure(error),
                }
            }
        },
        // Les huit messages d'édition écrivent dans le **brouillon**, jamais dans
        // l'instantané : seul `SettingsSaved(Ok(settings))` recopie la valeur réellement
        // persistée.
        _ => unreachable!("message routé vers un domaine incorrect"),
    }
    Task::none()
}
