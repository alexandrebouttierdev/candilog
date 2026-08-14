//! Transitions d'état du domaine `interactions`.

use super::*;

pub(super) fn handles(message: &Message) -> bool {
    matches!(
        message,
        Message::CandidatePressed(..)
            | Message::CandidateMoved(..)
            | Message::CandidateReleased
            | Message::CandidateCardHovered(..)
            | Message::CandidateCardExited
            | Message::CandidateDragHovered(..)
            | Message::CandidateDropped(..)
            | Message::CandidateDragCancelled
            | Message::OpenCandidateFromStats(..)
            | Message::CalendarViewChanged(..)
            | Message::CalendarDateSelected(..)
            | Message::ProfileFirstNameChanged(..)
            | Message::ProfileLastNameChanged(..)
            | Message::ProfileEmailChanged(..)
            | Message::ProfilePhoneChanged(..)
            | Message::ProfileCityChanged(..)
            | Message::ProfileHeadlineChanged(..)
            | Message::ProfileLinkedinChanged(..)
            | Message::ProfileGithubChanged(..)
            | Message::ProfileWebsiteChanged(..)
            | Message::ProfileSummaryChanged(..)
            | Message::ProfileSkillsChanged(..)
            | Message::ProfileSkillAdded
            | Message::ProfileSkillRemoved(..)
            | Message::ProfileItemAdded(..)
            | Message::ProfileItemRemoved(..)
            | Message::ProfileItemChanged(..)
            | Message::SubmitProfile
            | Message::DownloadUpdate
    )
}

pub(super) fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
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
        Message::OpenCandidateFromStats(id) => {
            if let Some(candidate) = app
                .data
                .follow_up_candidates
                .iter()
                .find(|candidate| candidate.id == id)
            {
                app.search.clone_from(&candidate.poste);
            }
            // Une relance reste ouvrable même si des filtres incompatibles étaient encore
            // actifs sur le suivi des candidatures. La recherche ciblée suffit à retrouver
            // la candidature après le rechargement paginé.
            app.candidate_filters = super::super::state::CandidateFilters::default();
            app.route = crate::navigation::Route::Candidatures;
            app.candidate_page = 1;
            app.dialog = Some(Dialog::CandidatureDetail(id));
            return recharger(app);
        }
        Message::CalendarViewChanged(view) => app.calendar_view = view,
        Message::CalendarDateSelected(date) => {
            app.calendar_date = date;
            app.calendar_year = date.year();
            app.calendar_month = date.month();
            app.calendar_view = super::super::message::CalendarView::Day;
        }
        Message::ProfileFirstNameChanged(value) => app.profile_personal_form.first_name = value,
        Message::ProfileLastNameChanged(value) => app.profile_personal_form.last_name = value,
        Message::ProfileEmailChanged(value) => app.profile_personal_form.email = value,
        Message::ProfilePhoneChanged(value) => app.profile_personal_form.phone = optional(&value),
        Message::ProfileCityChanged(value) => app.profile_personal_form.city = optional(&value),
        Message::ProfileHeadlineChanged(value) => {
            app.profile_personal_form.headline = optional(&value)
        }
        Message::ProfileLinkedinChanged(value) => {
            app.profile_personal_form.linkedin = optional(&value)
        }
        Message::ProfileGithubChanged(value) => app.profile_personal_form.github = optional(&value),
        Message::ProfileWebsiteChanged(value) => {
            app.profile_personal_form.website = optional(&value)
        }
        Message::ProfileSummaryChanged(action) => app.profile_summary_editor.perform(action),
        Message::ProfileSkillsChanged(value) => app.profile_skills_form = value,
        Message::ProfileSkillAdded => {
            let value = app.profile_skills_form.trim();
            if !value.is_empty()
                && !app
                    .profile_draft
                    .skills
                    .iter()
                    .any(|skill| skill.name.eq_ignore_ascii_case(value))
            {
                app.profile_draft
                    .skills
                    .push(crate::shared::profile::Skill {
                        name: value.to_owned(),
                    });
                app.profile_skills_form.clear();
            }
        }
        Message::ProfileSkillRemoved(index) => {
            if index < app.profile_draft.skills.len() {
                app.profile_draft.skills.remove(index);
            }
        }
        Message::ProfileItemAdded(collection) => {
            add_profile_item(&mut app.profile_draft, collection);
        }
        Message::ProfileItemRemoved(collection, index) => {
            remove_profile_item(&mut app.profile_draft, collection, index);
        }
        Message::ProfileItemChanged(collection, index, field, value) => {
            update_profile_item(&mut app.profile_draft, collection, index, field, value);
        }
        Message::SubmitProfile => {
            let mut profile = app.profile_draft.clone();
            profile.personal = app.profile_personal_form.clone();
            profile.personal.summary = optional(&app.profile_summary_editor.text());
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
        _ => unreachable!("message routé vers un domaine incorrect"),
    }
    Task::none()
}
