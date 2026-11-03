//! Import du profil depuis un CV : extraction, reprise, recadrage et progression.

use super::*;

impl AiService {
    pub async fn import_profile(
        &self,
        request: ProfileImportRequest,
        path: PathBuf,
        notifier: impl Fn(ProfileImportProgress) + Send + Sync,
    ) -> AppResult<AiExecution<ProfileImportAnalysis>> {
        let started_at = std::time::Instant::now();
        let id = request.generation_id.clone();
        let token = self.start(&id);
        let _guard = GenerationEnCours {
            service: self,
            id: id.clone(),
            token: Arc::clone(&token),
        };
        tracing::info!(method = ?request.method, "extraction de CV démarrée");

        let (config, routed) = load_task_config(&self.pool, AiTask::ImportResume)?;
        let provider = self.build(config.clone(), routed).await?;
        let model = effective_model_label(&config, routed, self)?;
        let reported = provider.reported_capabilities().await.ok().flatten();
        let capabilities = detect_model_capabilities(&config.provider, &model, reported.as_deref());
        let plan = resolve_cv_analysis_plan(request.method, capabilities);
        tracing::info!(
            primary = ?plan.primary,
            allow_fallback = plan.allow_text_fallback,
            vision_capable = capabilities.vision,
            "plan d'analyse de CV résolu"
        );

        let (profile, tokens, method_used, fallback_used) = match plan.primary {
            CvAnalysisMethodUsed::Vision => {
                match self
                    .extract_profile_via_vision(
                        path.clone(),
                        Arc::clone(&provider),
                        &token,
                        &id,
                        &notifier,
                    )
                    .await
                {
                    Ok((profile, tokens)) => (profile, tokens, CvAnalysisMethodUsed::Vision, false),
                    Err(AppError::Cancelled) => return Err(AppError::Cancelled),
                    Err(error) if plan.allow_text_fallback => {
                        tracing::warn!(
                            error = %error,
                            "extraction Vision échouée — repli sur l'analyse Texte"
                        );
                        emit_import(
                            &notifier,
                            &id,
                            Some("Repli sur l'analyse Texte…"),
                            "L'analyse visuelle a échoué. Candilog a poursuivi automatiquement avec l'analyse du texte.",
                            None,
                            None,
                        );
                        let (profile, tokens) = self
                            .extract_profile_via_text(path, provider, &token, &id, &notifier)
                            .await?;
                        (profile, tokens, CvAnalysisMethodUsed::Text, true)
                    }
                    Err(error) => {
                        emit_import(&notifier, &id, None, "Analyse du CV impossible", None, None);
                        return Err(error);
                    }
                }
            }
            CvAnalysisMethodUsed::Text => {
                let (profile, tokens) = self
                    .extract_profile_via_text(path, provider, &token, &id, &notifier)
                    .await?;
                (profile, tokens, CvAnalysisMethodUsed::Text, false)
            }
        };

        emit_detected(&notifier, &id, &profile, tokens);
        let message = if fallback_used {
            "Analyse terminée (repli Texte)"
        } else {
            "Analyse terminée"
        };
        emit_import(
            &notifier,
            &id,
            Some("Préparation de la revue…"),
            message,
            tokens,
            None,
        );
        tracing::info!(?method_used, fallback_used, "extraction de CV terminée");
        let current = self.profile()?;
        Ok(execution(
            started_at,
            ProfileImportAnalysis {
                preview: build_preview(&current, &profile),
                method_used,
                fallback_used,
            },
            tokens,
        ))
    }

    pub(super) async fn extract_profile_via_text(
        &self,
        path: PathBuf,
        provider: Arc<dyn LlmGenerator>,
        token: &CancellationToken,
        id: &str,
        notifier: &(impl Fn(ProfileImportProgress) + Send + Sync),
    ) -> AppResult<(Profile, Option<u32>)> {
        emit_import(
            notifier,
            id,
            Some("Lecture du fichier…"),
            "Lecture du fichier",
            None,
            None,
        );
        let text = match extract_pdf(path).await {
            Ok(text) => text,
            Err(error) => {
                emit_import(
                    notifier,
                    id,
                    None,
                    "Lecture du fichier impossible",
                    None,
                    None,
                );
                return Err(error);
            }
        };
        emit_import(
            notifier,
            id,
            Some("Extraction du contenu…"),
            "Texte extrait",
            None,
            None,
        );
        if let Err(error) = validate_source_text(&text, "Le CV") {
            emit_import(
                notifier,
                id,
                None,
                "Extraction du contenu impossible",
                None,
                None,
            );
            return Err(error);
        }
        emit_import(
            notifier,
            id,
            Some("Analyse du CV…"),
            "Analyse texte démarrée",
            None,
            None,
        );
        let analysis_text = truncate_chars(&text, 12_000);
        let (profile, tokens, _) = run_profile_pipeline(
            provider,
            token,
            Some((id, notifier)),
            &analysis_text,
            None,
            false,
        )
        .await?;
        Ok((profile, tokens))
    }

    pub(super) async fn extract_profile_via_vision(
        &self,
        path: PathBuf,
        provider: Arc<dyn LlmGenerator>,
        token: &CancellationToken,
        id: &str,
        notifier: &(impl Fn(ProfileImportProgress) + Send + Sync),
    ) -> AppResult<(Profile, Option<u32>)> {
        emit_import(
            notifier,
            id,
            Some("Lecture du fichier…"),
            "Lecture du fichier",
            None,
            None,
        );
        let complementary = try_extract_pdf_text(path.clone()).await?;
        if complementary.is_some() {
            emit_import(
                notifier,
                id,
                Some("Préparation des pages…"),
                "Texte complémentaire extrait",
                None,
                None,
            );
        }
        emit_import(
            notifier,
            id,
            Some("Conversion du PDF en images…"),
            "Préparation des images",
            None,
            None,
        );
        let pages = match cancel(token, render_pdf_pages(path)).await {
            Ok(pages) => pages,
            Err(AppError::Cancelled) => return Err(AppError::Cancelled),
            Err(error) => {
                emit_import(
                    notifier,
                    id,
                    None,
                    "Conversion du PDF impossible",
                    None,
                    None,
                );
                return Err(error);
            }
        };
        emit_import(
            notifier,
            id,
            Some("Analyse visuelle du CV…"),
            &format!(
                "{} page{} prête{}",
                pages.len(),
                if pages.len() > 1 { "s" } else { "" },
                if pages.len() > 1 { "s" } else { "" }
            ),
            None,
            None,
        );
        tracing::info!(pages = pages.len(), "PDF rendu — requête Vision");
        let images: Vec<VisionImage> = pages
            .into_iter()
            .map(|page| VisionImage {
                mime: page.mime,
                bytes: page.bytes,
            })
            .collect();
        let analysis_text = complementary
            .as_deref()
            .map(|text| truncate_chars(text, 12_000))
            .unwrap_or_default();
        let (profile, tokens, _) = run_profile_pipeline(
            provider,
            token,
            Some((id, notifier)),
            &analysis_text,
            Some(&images),
            true,
        )
        .await?;
        Ok((profile, tokens))
    }
}

/// Boucle d'extraction profil (texte ou vision) avec reprise sur gabarit vide.
pub(super) async fn run_profile_pipeline(
    provider: Arc<dyn LlmGenerator>,
    token: &CancellationToken,
    progress: Option<(&str, &(dyn Fn(ProfileImportProgress) + Send + Sync))>,
    analysis_text: &str,
    images: Option<&[VisionImage]>,
    vision: bool,
) -> AppResult<(Profile, Option<u32>, u32)> {
    let mut tokens = Some(0_u32);
    let mut llm_calls = 0_u32;
    for tentative in 0..2 {
        llm_calls += 1;
        if tentative > 0 {
            if let Some((id, notifier)) = progress {
                emit_import(
                    notifier,
                    id,
                    Some("Nouvel essai d'analyse…"),
                    if vision {
                        "Analyse visuelle sans résultat, nouvel essai"
                    } else {
                        "Analyse sans résultat, nouvel essai"
                    },
                    tokens,
                    None,
                );
            }
        }
        let invite = if vision {
            invite_import_vision(analysis_text, tentative > 0)
        } else {
            invite_import(analysis_text, tentative > 0)
        };
        let generation = if vision {
            let images = images.ok_or_else(|| {
                AppError::Validation("Aucune image fournie pour l'analyse visuelle.".into())
            })?;
            cancel_avec_progression(
                token,
                generate_json_vision(
                    Arc::clone(&provider),
                    &invite,
                    PROFILE_SYSTEM_VISION,
                    images,
                ),
                || {},
            )
            .await
        } else {
            cancel_avec_progression(
                token,
                generate_json(Arc::clone(&provider), &invite, PROFILE_SYSTEM),
                || {},
            )
            .await
        };
        let (mut candidat, appel): (Profile, Option<u32>) = match generation {
            Ok(sortie) => sortie,
            Err(AppError::Cancelled) => return Err(AppError::Cancelled),
            Err(error) => {
                if let Some((id, notifier)) = progress {
                    if !vision {
                        emit_import(notifier, id, None, "Analyse du CV impossible", None, None);
                    }
                }
                return Err(error);
            }
        };
        tokens = add_tokens(tokens, appel);
        normalize_profile_dates(&mut candidat);
        completer_descriptions_depuis_libelles(&mut candidat);
        if vision {
            // En Vision, le texte PDF n'est qu'un complément partiel : on y recadre
            // identité et libellés courts, mais on conserve les descriptions lues sur
            // les images (missions, projets, certifications) et le prénom / nom.
            if !analysis_text.trim().is_empty() {
                ground_imported_profile_keep_free_text(analysis_text, &mut candidat);
                completer_identite_noms(analysis_text, &mut candidat);
                completer_contacts_vides(analysis_text, &mut candidat);
                completer_formations_manquantes(analysis_text, &mut candidat);
            } else {
                completer_identite_noms("", &mut candidat);
            }
        } else {
            ground_imported_profile(analysis_text, &mut candidat);
            completer_identite_noms(analysis_text, &mut candidat);
            completer_contacts_vides(analysis_text, &mut candidat);
            completer_formations_manquantes(analysis_text, &mut candidat);
        }
        nettoyer_profile(&mut candidat);
        if !profil_vide(&candidat) {
            return Ok((candidat, tokens, llm_calls));
        }
    }
    if let Some((id, notifier)) = progress {
        emit_import(
            notifier,
            id,
            None,
            "Aucune donnée exploitable",
            tokens,
            None,
        );
    }
    Err(AppError::Provider(
        "Le modèle n'a extrait aucune information de ce CV. Réessayez, ou choisissez \
         un modèle plus grand : les plus petits n'y parviennent pas toujours."
            .into(),
    ))
}

pub(super) fn effective_model_label(
    config: &LlmConfig,
    routed: bool,
    service: &AiService,
) -> AppResult<String> {
    // Une route locale nomme déjà son modèle ; sans route, l'IA locale sert le modèle actif.
    if matches!(config.provider, ProviderKind::CandilogLocal) && !routed {
        if let Ok(Some(tag)) = service.managed_ollama.active_ollama_tag() {
            if !tag.trim().is_empty() {
                return Ok(tag);
            }
        }
    }
    if config.model.trim().is_empty() {
        Ok("Modèle actif".into())
    } else {
        Ok(config.model.clone())
    }
}

/// Invite d'analyse d'un CV, relancée en disant ce qui manquait à la réponse précédente.
pub(super) fn invite_import(analysis_text: &str, reprise: bool) -> String {
    let bloc = bloc_donnees("cv", analysis_text);
    if reprise {
        format!(
            "{bloc}\n\nLa réponse précédente ne contenait aucune information : elle recopiait \
             le gabarit au lieu de le remplir. Renseigne chaque champ avec les valeurs lues \
             dans le CV ci-dessus."
        )
    } else {
        bloc
    }
}

pub(super) fn invite_import_vision(analysis_text: &str, reprise: bool) -> String {
    let mut parts = vec!["Analyse les images du CV fournies (source principale).".to_owned()];
    if !analysis_text.trim().is_empty() {
        parts.push(bloc_donnees("cv_texte", analysis_text));
        parts.push(
            "Le texte ci-dessus est un complément optionnel. Privilegie le document visuel.".into(),
        );
    }
    if reprise {
        parts.push(
            "La réponse précédente ne contenait aucune information. Remplis chaque champ \
             depuis les images du CV."
                .into(),
        );
    }
    parts.join("\n\n")
}

/// Un profil sans identité, sans expérience et sans compétence n'a rien d'exploitable.
pub(super) fn profil_vide(profile: &Profile) -> bool {
    profile.identity.first_name.trim().is_empty()
        && profile.identity.name.trim().is_empty()
        && profile.experiences.is_empty()
        && profile.skills.is_empty()
}

pub(super) fn emit_import(
    notifier: &(impl Fn(ProfileImportProgress) + ?Sized),
    id: &str,
    step: Option<&str>,
    message: &str,
    tokens_used: Option<u32>,
    tokens_per_second: Option<f32>,
) {
    notifier(ProfileImportProgress {
        generation_id: id.into(),
        at: chrono::Utc::now().to_rfc3339(),
        message: message.into(),
        step: step.map(str::to_owned),
        tokens_used,
        tokens_per_second,
    });
}

pub(super) fn emit_detected(
    notifier: &(impl Fn(ProfileImportProgress) + ?Sized),
    id: &str,
    profile: &Profile,
    tokens_used: Option<u32>,
) {
    let lines = [
        counted(
            profile.experiences.len(),
            "expérience détectée",
            "expériences détectées",
        ),
        counted(
            profile.skills.len(),
            "compétence détectée",
            "compétences détectées",
        ),
        counted(
            profile.education.len(),
            "formation détectée",
            "formations détectées",
        ),
        counted(
            profile.languages.len(),
            "langue détectée",
            "langues détectées",
        ),
        counted(profile.projects.len(), "projet détecté", "projets détectés"),
        counted(
            profile.certifications.len(),
            "certification détectée",
            "certifications détectées",
        ),
    ];
    for line in lines.into_iter().flatten() {
        emit_import(notifier, id, None, &line, tokens_used, None);
    }
}

pub(super) fn counted(count: usize, singular: &str, plural: &str) -> Option<String> {
    match count {
        0 => None,
        1 => Some(format!("1 {singular}")),
        n => Some(format!("{n} {plural}")),
    }
}
pub(super) fn decouper_fragments(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0;
    for (index, caractere) in text.char_indices() {
        if matches!(caractere, '.' | '!' | '?' | '\n') && index + caractere.len_utf8() > start {
            out.push(text[start..index + caractere.len_utf8()].to_owned());
            start = index + caractere.len_utf8();
        }
    }
    if start < text.len() {
        out.push(text[start..].to_owned());
    }
    out.into_iter().filter(|v| !v.is_empty()).collect()
}
pub(super) fn nettoyer_profile(profile: &mut Profile) {
    profile
        .experiences
        .retain(|v| !v.title.trim().is_empty() || !v.company.trim().is_empty());
    profile.skills.retain(|v| !v.name.trim().is_empty());
    profile
        .education
        .retain(|v| !v.degree.trim().is_empty() || !v.school.trim().is_empty());
    profile.languages.retain(|v| !v.name.trim().is_empty());
    profile.projects.retain(|v| !v.name.trim().is_empty());
    profile.certifications.retain(|v| !v.name.trim().is_empty());
    profile.interests.retain(|v| !v.name.trim().is_empty());
    for experience in &mut profile.experiences {
        if experience.current {
            experience.end_date = None;
        }
    }
}
