//! Analyse d'un CV importé face à une offre, par texte ou par images.

use super::*;

impl AiService {
    pub async fn analyze_resume_imported(
        &self,
        request: ResumeAnalysisRequest,
        notifier: impl Fn(AiProgress),
    ) -> AppResult<AiExecution<ImportedResumeAnalysis>> {
        let started_at = std::time::Instant::now();
        validate_source_text(&request.job_offer, "L'offre")?;
        let id = request.generation_id.clone();
        let token = self.start(&id);
        let _guard = GenerationEnCours {
            service: self,
            id: id.clone(),
            token: Arc::clone(&token),
        };

        let path = self.selected_resume_path()?;
        progres(&notifier, &id, "Lecture locale du PDF", None, None);
        let text = match try_extract_pdf_text(path.clone()).await? {
            Some(extracted) if !extracted.trim().is_empty() => extracted,
            _ => extract_pdf(path.clone()).await?,
        };
        validate_source_text(&text, "Le CV")?;

        let (config, routed) = load_task_config(&self.pool, AiTask::AnalyzeResume)?;
        let provider = self.build(config.clone(), routed).await?;
        let model = effective_model_label(&config, routed, self)?;
        let reported = provider.reported_capabilities().await.ok().flatten();
        let capabilities = detect_model_capabilities(&config.provider, &model, reported.as_deref());
        let plan = resolve_cv_analysis_plan(request.method, capabilities);
        tracing::info!(
            primary = ?plan.primary,
            allow_fallback = plan.allow_text_fallback,
            vision_capable = capabilities.vision,
            "plan d'analyse ATS CV résolu"
        );

        let mut tokens = Some(0_u32);
        let (mut resume, method_used, fallback_used) = match plan.primary {
            CvAnalysisMethodUsed::Vision => {
                match self
                    .parse_resume_via_vision(
                        path.clone(),
                        &text,
                        Arc::clone(&provider),
                        &token,
                        &id,
                        &notifier,
                    )
                    .await
                {
                    Ok((resume, call_tokens)) => {
                        tokens = add_tokens(tokens, call_tokens);
                        (resume, CvAnalysisMethodUsed::Vision, false)
                    }
                    Err(AppError::Cancelled) => return Err(AppError::Cancelled),
                    Err(error) if plan.allow_text_fallback => {
                        tracing::warn!(
                            error = %error,
                            "structuration Vision échouée — repli Texte (analyse ATS)"
                        );
                        progres(&notifier, &id, "Repli sur l'analyse Texte…", None, tokens);
                        let (resume, call_tokens) = self
                            .parse_resume_via_text(
                                &text,
                                Arc::clone(&provider),
                                &token,
                                &id,
                                &notifier,
                            )
                            .await?;
                        tokens = add_tokens(tokens, call_tokens);
                        (resume, CvAnalysisMethodUsed::Text, true)
                    }
                    Err(error) => return Err(error),
                }
            }
            CvAnalysisMethodUsed::Text => {
                let (resume, call_tokens) = self
                    .parse_resume_via_text(&text, Arc::clone(&provider), &token, &id, &notifier)
                    .await?;
                tokens = add_tokens(tokens, call_tokens);
                (resume, CvAnalysisMethodUsed::Text, false)
            }
        };

        ground_imported_resume(&text, &mut resume);
        progres(&notifier, &id, "Analyse de l'offre", None, tokens);
        let (mut job_offer, call_tokens): (StructuredListing, Option<u32>) = cancel(
            &token,
            generate_json(
                provider.clone(),
                &bloc_donnees("offre", &request.job_offer),
                JOB_OFFER_SYSTEM,
            ),
        )
        .await?;
        tokens = add_tokens(tokens, call_tokens);
        ground_extracted_listing(&request.job_offer, &mut job_offer);
        // Score déterministe sur le PDF brut (pas seulement le JSON LLM).
        let score = score_resume_imported_with_source(&resume, &job_offer, Some(&text));
        progres(&notifier, &id, "Recommandations ATS", None, tokens);
        let (mut analysis, call_tokens): (AtsAnalysis, Option<u32>) = cancel(
            &token,
            generate_json(
                provider,
                &bloc_donnees(
                    "analyse",
                    &serde_json::json!({
                        "cv": resume,
                        "cv_texte": text,
                        "offre": job_offer,
                        "score_candilog": score,
                        "method_used": method_used,
                    })
                    .to_string(),
                ),
                ATS_SYSTEM,
            ),
        )
        .await?;
        ground_ats_recommendations(&resume, &score, &mut analysis);
        tokens = add_tokens(tokens, call_tokens);
        progres(&notifier, &id, "Terminé", None, tokens);
        Ok(execution(
            started_at,
            ImportedResumeAnalysis {
                resume,
                job_offer,
                score,
                analysis,
                method_used,
                fallback_used,
            },
            tokens,
        ))
    }

    pub(super) async fn parse_resume_via_text(
        &self,
        text: &str,
        provider: Arc<dyn LlmGenerator>,
        token: &CancellationToken,
        id: &str,
        notifier: &impl Fn(AiProgress),
    ) -> AppResult<(GeneratedResume, Option<u32>)> {
        progres(notifier, id, "Structuration du CV (texte)", None, None);
        cancel(
            token,
            generate_json(provider, &bloc_donnees("cv", text), PARSE_RESUME_SYSTEM),
        )
        .await
    }

    pub(super) async fn parse_resume_via_vision(
        &self,
        path: PathBuf,
        complementary_text: &str,
        provider: Arc<dyn LlmGenerator>,
        token: &CancellationToken,
        id: &str,
        notifier: &impl Fn(AiProgress),
    ) -> AppResult<(GeneratedResume, Option<u32>)> {
        progres(notifier, id, "Conversion du PDF en images…", None, None);
        let pages = cancel(token, render_pdf_pages(path)).await?;
        progres(
            notifier,
            id,
            &format!(
                "Analyse visuelle du CV ({} page{})…",
                pages.len(),
                if pages.len() > 1 { "s" } else { "" }
            ),
            None,
            None,
        );
        let images: Vec<VisionImage> = pages
            .into_iter()
            .map(|page| VisionImage {
                mime: page.mime,
                bytes: page.bytes,
            })
            .collect();
        let prompt = format!(
            "{}

{}",
            bloc_donnees(
                "cv_texte_complementaire",
                &truncate_chars(complementary_text, 12_000)
            ),
            "Structure le CV à partir des images fournies."
        );
        cancel(
            token,
            generate_json_vision::<GeneratedResume>(
                provider,
                &prompt,
                PARSE_RESUME_SYSTEM_VISION,
                &images,
            ),
        )
        .await
    }

    /// Capacités du modèle actuellement configuré (pour l'UI d'import).
    pub async fn active_model_capabilities(&self) -> AppResult<ActiveModelCapabilities> {
        // L'écran d'import interroge le modèle qui lira effectivement le CV.
        let (config, routed) = load_task_config(&self.pool, AiTask::ImportResume)?;
        let provider = self.build(config.clone(), routed).await?;
        let reported = provider.reported_capabilities().await.ok().flatten();
        let model = effective_model_label(&config, routed, self)?;
        let capabilities = detect_model_capabilities(&config.provider, &model, reported.as_deref());
        Ok(ActiveModelCapabilities {
            vision: capabilities.vision,
            provider_label: provider_label(&config),
            model_label: model,
        })
    }
}
