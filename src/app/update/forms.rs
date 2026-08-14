//! Transitions d'état du domaine `forms`.

use super::*;

pub(super) fn handles(message: &Message) -> bool {
    matches!(
        message,
        Message::EntrepriseNomChanged(..)
            | Message::EntrepriseSecteurChanged(..)
            | Message::EntrepriseTypeChanged(..)
            | Message::EntrepriseSiteChanged(..)
            | Message::EntrepriseVilleChanged(..)
            | Message::EntrepriseAdresseChanged(..)
            | Message::EntrepriseNotesChanged(..)
            | Message::SubmitEntreprise
            | Message::ContactPrenomChanged(..)
            | Message::ContactNomChanged(..)
            | Message::ContactPosteChanged(..)
            | Message::ContactEmailChanged(..)
            | Message::ContactTelephoneChanged(..)
            | Message::ContactLinkedinChanged(..)
            | Message::ContactNotesChanged(..)
            | Message::ContactEntrepriseChanged(..)
            | Message::SubmitContact
            | Message::CandidaturePosteChanged(..)
            | Message::CandidatureEntrepriseChanged(..)
            | Message::CandidatureContratChanged(..)
            | Message::CandidatureStatutChanged(..)
            | Message::CandidatureDateChanged(..)
            | Message::CandidatureLienChanged(..)
            | Message::CandidatureNotesChanged(..)
            | Message::SubmitCandidature
            | Message::MoveCandidature(..)
            | Message::CandidatureStatusUpdated(..)
            | Message::EntretienCandidatureChanged(..)
            | Message::EntretienContactChanged(..)
            | Message::EntretienDateChanged(..)
            | Message::EntretienTypeChanged(..)
            | Message::EntretienLieuChanged(..)
            | Message::EntretienNotesChanged(..)
            | Message::EntretienCompteRenduChanged(..)
            | Message::SubmitEntretien
            | Message::RelanceCandidatureChanged(..)
            | Message::RelanceDateChanged(..)
            | Message::RelanceTypeChanged(..)
            | Message::RelanceNotesChanged(..)
            | Message::SubmitRelance
            | Message::OpenDatePicker(..)
            | Message::CloseDatePicker
            | Message::DatePickerPreviousMonth
            | Message::DatePickerNextMonth
            | Message::DatePickerSelected(..)
            | Message::ExportCandidatures
            | Message::CandidaturesExported(..)
    )
}

pub(super) fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::EntrepriseNomChanged(value) => app.entreprise_form.nom = value,
        Message::EntrepriseSecteurChanged(value) => app.entreprise_form.secteur = value,
        Message::EntrepriseTypeChanged(value) => app.entreprise_form.type_ = value,
        Message::EntrepriseSiteChanged(value) => app.entreprise_form.site_web = value,
        Message::EntrepriseVilleChanged(value) => app.entreprise_form.ville = value,
        Message::EntrepriseAdresseChanged(value) => app.entreprise_form.adresse = value,
        Message::EntrepriseNotesChanged(action) => app.entreprise_form.notes.perform(action),
        Message::SubmitEntreprise => {
            let input = NouvelleEntreprise {
                nom: app.entreprise_form.nom.clone(),
                secteur: optional(&app.entreprise_form.secteur),
                type_: optional(&app.entreprise_form.type_),
                site_web: optional(&app.entreprise_form.site_web),
                ville: optional(&app.entreprise_form.ville),
                adresse: optional(&app.entreprise_form.adresse),
                notes: optional(&app.entreprise_form.notes.text()),
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
        Message::ContactNotesChanged(action) => app.contact_form.notes.perform(action),
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
                notes: optional(&app.contact_form.notes.text()),
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
        Message::EntretienNotesChanged(action) => app.entretien_form.notes.perform(action),
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
            let notes = app.entretien_form.notes.text();
            let compte_rendu = app.entretien_form.compte_rendu.text();
            let input = NouvelEntretien {
                candidature_id,
                contact_id: app.entretien_form.contact_id,
                date_entretien,
                type_entretien: app.entretien_form.type_entretien,
                lieu: optional(&app.entretien_form.lieu),
                notes: optional(&notes),
                compte_rendu: optional(&compte_rendu),
            };
            let edition = app.editing_id;
            return ecrire(app, "Entretien enregistré.", move |backend| {
                backend
                    .entretiens
                    .enregistrer_avec_statut(edition, &input)
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
        Message::OpenDatePicker(target) => {
            let value = match target {
                DatePickerTarget::Candidature => &app.candidature_form.date_envoi,
                DatePickerTarget::Entretien => &app.entretien_form.date_entretien,
                DatePickerTarget::Relance => &app.relance_form.date_relance,
                DatePickerTarget::FiltreDebut => &app.candidate_filters.date_from,
                DatePickerTarget::FiltreFin => &app.candidate_filters.date_to,
            };
            let date_text = value.get(..10).unwrap_or(value);
            let selected = chrono::NaiveDate::parse_from_str(date_text, "%d-%m-%Y")
                .unwrap_or_else(|_| Local::now().date_naive());
            app.date_picker = Some(DatePickerState {
                target,
                year: selected.year(),
                month: selected.month(),
            });
        }
        Message::CloseDatePicker => app.date_picker = None,
        Message::DatePickerPreviousMonth => {
            if let Some(picker) = app.date_picker.as_mut() {
                if picker.month == 1 {
                    picker.month = 12;
                    picker.year -= 1;
                } else {
                    picker.month -= 1;
                }
            }
        }
        Message::DatePickerNextMonth => {
            if let Some(picker) = app.date_picker.as_mut() {
                if picker.month == 12 {
                    picker.month = 1;
                    picker.year += 1;
                } else {
                    picker.month += 1;
                }
            }
        }
        Message::DatePickerSelected(date) => {
            let Some(target) = app.date_picker.take().map(|picker| picker.target) else {
                return Task::none();
            };
            let formatted = date.format("%d-%m-%Y").to_string();
            match target {
                DatePickerTarget::Candidature => app.candidature_form.date_envoi = formatted,
                DatePickerTarget::Entretien => {
                    let time = app
                        .entretien_form
                        .date_entretien
                        .split_once(' ')
                        .map_or("09:00", |(_, time)| time);
                    app.entretien_form.date_entretien = format!("{formatted} {time}");
                }
                DatePickerTarget::Relance => app.relance_form.date_relance = formatted,
                DatePickerTarget::FiltreDebut => {
                    app.candidate_filters.date_from = formatted;
                    app.candidate_page = 1;
                    return recharger(app);
                }
                DatePickerTarget::FiltreFin => {
                    app.candidate_filters.date_to = formatted;
                    app.candidate_page = 1;
                    return recharger(app);
                }
            }
        }
        Message::ExportCandidatures => {
            let Some(backend) = app.backend.clone() else {
                app.notify_failure("La base Candilog n'est pas disponible.");
                return Task::none();
            };
            let query = app.snapshot_request().candidate_query();
            return Task::perform(
                async move {
                    let rows = tokio::task::spawn_blocking(move || {
                        backend
                            .candidatures
                            .lister_page(1, u64::MAX, &query)
                            .map(|page| page.items)
                            .map_err(|error| error.to_string())
                    })
                    .await
                    .map_err(|error| format!("Export interrompu : {error}"))??;
                    export_candidatures(rows).await
                },
                Message::CandidaturesExported,
            );
        }
        Message::CandidaturesExported(result) => match result {
            Ok(path) => app.notify_success(format!("Export créé : {}", path.display())),
            Err(error) => app.notify_failure(error),
        },
        _ => unreachable!("message routé vers un domaine incorrect"),
    }
    Task::none()
}
