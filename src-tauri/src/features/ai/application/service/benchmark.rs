//! Phrase de test du modèle local et benchmark utilisateur sur le CV de référence.

use super::*;

/// Envoie la phrase de test et mesure l'aller-retour.
pub(super) async fn probe(
    provider: &dyn LlmGenerator,
    model: String,
) -> AppResult<LocalModelProbe> {
    let started = std::time::Instant::now();
    let output = provider.generate(PROBE_PROMPT, PROBE_SYSTEM, false).await?;
    if output.text.trim().is_empty() {
        return Err(AppError::Provider(
            "Le modèle local n'a rien répondu à la phrase de test.".into(),
        ));
    }
    Ok(LocalModelProbe {
        model,
        latency_ms: u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX),
    })
}

impl AiService {
    /// Phrase de test de l'installation locale : le modèle actif répond-il, et en combien
    /// de temps ? Un seul appel court, rien n'est enregistré.
    ///
    /// # Errors
    /// `Provider` si aucun modèle local n'est actif, si le moteur ne démarre pas ou si le
    /// modèle ne répond rien.
    pub async fn probe_local_model(&self) -> AppResult<LocalModelProbe> {
        let model = self
            .managed_ollama
            .active_ollama_tag()?
            .ok_or_else(|| AppError::Provider("Aucun modèle local n'est installé.".into()))?;
        let config = load_config(&self.pool)?;
        let provider = self
            .build(
                LlmConfig {
                    provider: ProviderKind::CandilogLocal,
                    api_key: None,
                    endpoint: None,
                    model: model.clone(),
                    temperature: config.temperature,
                    mode: config.mode,
                },
                false,
            )
            .await?;
        probe(provider.as_ref(), model).await
    }

    /// Benchmark utilisateur sur `CV_BENCHMARK.pdf` : pipeline réel, aucune persistance.
    pub async fn run_user_cv_benchmark(
        &self,
        request: UserBenchmarkRequest,
    ) -> AppResult<UserBenchmarkResult> {
        let ground_truth = load_ground_truth().map_err(AppError::Provider)?;
        let pdf_path = benchmark_pdf_path().map_err(|error| {
            tracing::error!(%error, "matérialisation du CV de benchmark impossible");
            AppError::Provider("Le CV de référence du benchmark est introuvable.".into())
        })?;
        let config = load_config(&self.pool)?;
        let remote_warning = !matches!(
            config.provider,
            ProviderKind::CandilogLocal | ProviderKind::Ollama
        );
        let generation_id = request.generation_id.clone();
        let started_at = std::time::Instant::now();
        let token = self.start(&generation_id);
        let _guard = GenerationEnCours {
            service: self,
            id: generation_id,
            token: Arc::clone(&token),
        };
        let provider = self.provider().await?;
        let model = effective_model_label(&config, false, self)?;
        let reported = provider.reported_capabilities().await.ok().flatten();
        let capabilities = detect_model_capabilities(&config.provider, &model, reported.as_deref());
        let plan = resolve_cv_analysis_plan(request.method, capabilities);

        let pdf_started = std::time::Instant::now();
        let mut preprocess_ms = 0_u32;
        let mut pdf_extract_ms;
        let mut fallback_used = false;
        let llm_started = std::time::Instant::now();

        let (profile, tokens, llm_calls, method_used) = match plan.primary {
            CvAnalysisMethodUsed::Vision => {
                let complementary = try_extract_pdf_text(pdf_path.clone()).await?;
                pdf_extract_ms = pdf_started.elapsed().as_millis() as u32;
                let render_started = std::time::Instant::now();
                match render_pdf_pages(pdf_path.clone()).await {
                    Ok(pages) => {
                        preprocess_ms = render_started.elapsed().as_millis() as u32;
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
                        match run_profile_pipeline(
                            Arc::clone(&provider),
                            &token,
                            None,
                            &analysis_text,
                            Some(&images),
                            true,
                        )
                        .await
                        {
                            Ok((profile, tokens, calls)) => {
                                (profile, tokens, calls, CvAnalysisMethodUsed::Vision)
                            }
                            Err(AppError::Cancelled) => return Err(AppError::Cancelled),
                            Err(error) if plan.allow_text_fallback => {
                                tracing::warn!(
                                    error = %error,
                                    "benchmark Vision échoué — repli Texte"
                                );
                                fallback_used = true;
                                let text = complementary.unwrap_or_default();
                                let text = if text.trim().is_empty() {
                                    extract_pdf(pdf_path).await?
                                } else {
                                    text
                                };
                                validate_source_text(&text, "Le CV de benchmark")?;
                                let analysis_text = truncate_chars(&text, 12_000);
                                let (profile, tokens, calls) = run_profile_pipeline(
                                    provider,
                                    &token,
                                    None,
                                    &analysis_text,
                                    None,
                                    false,
                                )
                                .await?;
                                (profile, tokens, calls, CvAnalysisMethodUsed::Text)
                            }
                            Err(error) => return Err(error),
                        }
                    }
                    Err(error) if plan.allow_text_fallback => {
                        tracing::warn!(
                            error = %error,
                            "benchmark Vision : rendu PDF échoué — repli Texte"
                        );
                        fallback_used = true;
                        let text = if let Some(text) = complementary {
                            text
                        } else {
                            extract_pdf(pdf_path).await?
                        };
                        pdf_extract_ms = pdf_started.elapsed().as_millis() as u32;
                        validate_source_text(&text, "Le CV de benchmark")?;
                        let analysis_text = truncate_chars(&text, 12_000);
                        let (profile, tokens, calls) = run_profile_pipeline(
                            provider,
                            &token,
                            None,
                            &analysis_text,
                            None,
                            false,
                        )
                        .await?;
                        (profile, tokens, calls, CvAnalysisMethodUsed::Text)
                    }
                    Err(error) => return Err(error),
                }
            }
            CvAnalysisMethodUsed::Text => {
                let text = extract_pdf(pdf_path).await?;
                pdf_extract_ms = pdf_started.elapsed().as_millis() as u32;
                validate_source_text(&text, "Le CV de benchmark")?;
                let analysis_text = truncate_chars(&text, 12_000);
                let (profile, tokens, calls) =
                    run_profile_pipeline(provider, &token, None, &analysis_text, None, false)
                        .await?;
                (profile, tokens, calls, CvAnalysisMethodUsed::Text)
            }
        };

        let llm_ms = llm_started.elapsed().as_millis() as u32;
        let parse_started = std::time::Instant::now();
        let score = score_extracted_profile(&ground_truth.profile, &profile);
        let parse_ms = parse_started.elapsed().as_millis() as u32;
        let total_ms = started_at.elapsed().as_millis() as u32;
        let provider_name = provider_label(&config);
        let result = build_benchmark_result(
            ground_truth.benchmark_version,
            score,
            UserBenchmarkMetrics {
                total_ms,
                pdf_extract_ms,
                preprocess_ms,
                llm_ms,
                parse_ms,
                llm_calls,
                tokens_input: None,
                tokens_output: tokens,
                tokens_per_second: if llm_ms > 0 {
                    tokens.map(|value| value as f32 / (llm_ms as f32 / 1000.0))
                } else {
                    None
                },
            },
            provider_name.clone(),
            model.clone(),
            remote_warning,
            BenchmarkAnalysisOutcome {
                method_used,
                fallback_used,
            },
        );
        self.managed_ollama
            .record_benchmark(StoredBenchmarkResult {
                provider: provider_name,
                model,
                benchmark_version: result.benchmark_version,
                score: result.score,
                total_ms: result.metrics.total_ms,
                measured_at: chrono::Utc::now().to_rfc3339(),
            })?;
        Ok(result)
    }
}
