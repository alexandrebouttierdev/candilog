//! Génération de CV et de lettre, analyse d'offre et relecture française.

use super::*;

impl AiService {
    pub async fn analyze_listing(&self, text: String) -> AppResult<AiExecution<ListingAnalysis>> {
        let started_at = std::time::Instant::now();
        validate_source_text(&text, "L'offre")?;
        let (mut job_offer, tokens): (StructuredListing, Option<u32>) = generate_json(
            self.provider_for(AiTask::ExtractOffer).await?,
            &bloc_donnees("offre", &text),
            JOB_OFFER_SYSTEM,
        )
        .await?;
        ground_extracted_listing(&text, &mut job_offer);
        let score = profile_score(&self.profile()?, &job_offer);
        Ok(execution(
            started_at,
            ListingAnalysis { job_offer, score },
            tokens,
        ))
    }

    /// Adéquation de la lettre à l'offre (`screens/16`), recalculée localement à chaque
    /// version : aucun appel au modèle, les recommandations ne viennent que du profil.
    ///
    /// # Errors
    /// `Validation` si la lettre ou l'offre transmises dépassent les bornes.
    pub fn evaluate_cover_letter(&self, request: LetterFitRequest) -> AppResult<LetterFit> {
        if request.letter.chars().count() > MAX_LETTER_FIT_CHARS {
            return Err(AppError::Validation(
                "La lettre est trop longue pour être évaluée.".into(),
            ));
        }
        request
            .job_offer
            .validate_ai_output()
            .map_err(|_| AppError::Validation("L'offre transmise est invalide.".into()))?;
        let profile = profile_without(&self.profile()?, &request.excluded_sections);
        Ok(letter_fit(
            &request.letter,
            &request.job_offer,
            &build_fact_catalog(&profile),
        ))
    }

    pub async fn generate_resume(
        &self,
        request: ResumeGenerationRequest,
        notifier: impl Fn(AiProgress),
    ) -> AppResult<AiExecution<ResumeGeneration>> {
        let started_at = std::time::Instant::now();
        validate_source_text(&request.job_offer, "L'offre")?;
        let id = request.generation_id.clone();
        let token = self.start(&id);
        let _guard = GenerationEnCours {
            service: self,
            id,
            token: Arc::clone(&token),
        };
        let result = self
            .generate_resume_interne(&request, &token, &notifier)
            .await;
        let (output, tokens_used) = match result {
            Ok(value) => value,
            Err(error)
                if matches!(
                    &error,
                    AppError::Http(_) | AppError::Provider(_) | AppError::Serialization(_)
                ) =>
            {
                tracing::warn!(error = %error, "génération de CV poursuivie en mode local");
                let profile = profile_without(&self.profile()?, &request.excluded_sections);
                validate_profile_input(&profile)?;
                progres(
                    &notifier,
                    &request.generation_id,
                    "Suggestions locales disponibles",
                    None,
                    None,
                );
                (
                    fallback_resume_generation(&profile, &request.job_offer, error.user_message()),
                    None,
                )
            }
            Err(error) => return Err(error),
        };
        Ok(execution(started_at, output, tokens_used))
    }

    pub(super) async fn generate_resume_interne(
        &self,
        request: &ResumeGenerationRequest,
        token: &CancellationToken,
        notifier: &impl Fn(AiProgress),
    ) -> AppResult<(ResumeGeneration, Option<u32>)> {
        let full_profile = self.profile()?;
        validate_profile_input(&full_profile)?;
        if full_profile.identity.first_name.trim().is_empty()
            && full_profile.experiences.is_empty()
            && full_profile.skills.is_empty()
        {
            return Err(AppError::Validation(
                "Complétez votre profil avant de générer un CV".into(),
            ));
        }
        // Les sections écartées par l'utilisateur ne quittent jamais la machine : elles sont
        // retirées avant le premier appel au modèle.
        let profile = profile_without(&full_profile, &request.excluded_sections);
        if profile.experiences.is_empty()
            && profile.education.is_empty()
            && profile.skills.is_empty()
            && profile.projects.is_empty()
        {
            return Err(AppError::Validation(
                "Laissez à l'IA au moins une section : expériences, formations, compétences ou projets".into(),
            ));
        }
        let provider = self.provider_for(AiTask::GenerateResume).await?;
        let mut tokens = Some(0_u32);
        progres(
            notifier,
            &request.generation_id,
            "Analyse de l'offre",
            None,
            None,
        );
        let (mut job_offer, call_tokens): (StructuredListing, Option<u32>) = cancel(
            token,
            generate_json(
                provider.clone(),
                &bloc_donnees("offre", &request.job_offer),
                JOB_OFFER_SYSTEM,
            ),
        )
        .await?;
        tokens = add_tokens(tokens, call_tokens);
        ground_extracted_listing(&request.job_offer, &mut job_offer);
        let score = profile_score(&profile, &job_offer);
        progres(
            notifier,
            &request.generation_id,
            "Adaptation du CV",
            None,
            tokens,
        );
        let context =
            serde_json::json!({"profile":profile,"offre":job_offer,"score":score}).to_string();
        let resume_system = format!("{RESUME_SYSTEM}\n{}", request.tone.instruction());
        let (mut resume, call_tokens): (GeneratedResume, Option<u32>) = cancel(
            token,
            generate_json(
                provider.clone(),
                &bloc_donnees("contexte", &context),
                &resume_system,
            ),
        )
        .await?;
        tokens = add_tokens(tokens, call_tokens);
        ground_generated_resume(&profile, &mut resume);
        progres(
            notifier,
            &request.generation_id,
            "Relecture du français",
            None,
            tokens,
        );
        let correction_request = resume_correction_request(&request.generation_id, &resume);
        if !correction_request.fields.is_empty() {
            let (correction, call_tokens) = cancel(
                token,
                correct_language_fields(provider.clone(), &correction_request),
            )
            .await?;
            tokens = add_tokens(tokens, call_tokens);
            apply_resume_correction(&mut resume, &correction);
        }
        // Le socle a déjà été recadré sur le profil avant la relecture. Le recadrer une
        // seconde fois restaurerait les textes source et annulerait toutes les corrections.
        progres(
            notifier,
            &request.generation_id,
            "Analyse ATS",
            None,
            tokens,
        );
        let content_catalog = profile_content_catalog(&profile);
        // Le socle éditorial ne porte volontairement aucun élément optionnel. Les compétences
        // sélectionnées par l'étape de rédaction deviennent des candidates, jamais des ajouts
        // silencieux au document.
        let mut resume_base = resume.clone();
        resume_base.skills.clear();
        let context_ats = serde_json::json!({
            "cv": resume_base,
            "offre": job_offer,
            "score_candilog": score,
            "contenu_profil": content_catalog,
        })
        .to_string();
        let (mut analysis, call_tokens): (AtsAnalysis, Option<u32>) = cancel(
            token,
            generate_json(
                provider.clone(),
                &bloc_donnees("analyse", &context_ats),
                ATS_SYSTEM,
            ),
        )
        .await?;
        ground_content_recommendations(&content_catalog, &mut analysis);
        ground_ats_recommendations(&resume, &score, &mut analysis);
        tokens = add_tokens(tokens, call_tokens);
        progres(notifier, &request.generation_id, "Terminé", None, tokens);
        Ok((
            ResumeGeneration {
                resume,
                analysis,
                job_offer,
                profile_score: score,
                recommendation_error: None,
            },
            tokens,
        ))
    }

    pub async fn generate_cover_letter(
        &self,
        request: CoverLetterRequest,
        notifier: impl Fn(AiProgress),
    ) -> AppResult<AiExecution<String>> {
        let started_at = std::time::Instant::now();
        validate_cover_letter_request(&request)?;
        if request
            .company
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
            && request
                .job_title
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
            && request
                .context
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
        {
            return Err(AppError::Validation(
                "Précisez une entreprise, un poste ou le contexte de la candidature".into(),
            ));
        }
        let id = request.generation_id.clone();
        let token = self.start(&id);
        let _guard = GenerationEnCours {
            service: self,
            id: id.clone(),
            token: Arc::clone(&token),
        };
        let profile = profile_without(&self.profile()?, &request.excluded_sections);
        validate_profile_input(&profile)?;
        let catalog = build_fact_catalog(&profile);
        if catalog.is_empty() && !request.excluded_sections.is_empty() {
            return Err(AppError::Validation(
                "Autorisez au moins un argument : la lettre n'a rien sur quoi s'appuyer".into(),
            ));
        }
        // Sur une itération, on compacte le brief : la lettre précédente + la consigne
        // suffisent à réorienter la sélection de faits, sans renvoyer toute l'offre.
        let iterating = request
            .previous_cover_letter
            .as_deref()
            .is_some_and(|letter| !letter.trim().is_empty());
        let compact_catalog: Vec<_> = catalog
            .iter()
            .map(|fact| {
                let text = if iterating {
                    truncate_chars(&fact.text, 220)
                } else {
                    truncate_chars(&fact.text, 480)
                };
                serde_json::json!({ "id": fact.id, "kind": fact.kind, "text": text })
            })
            .collect();
        let context = if iterating {
            serde_json::json!({
                "catalogue": compact_catalog,
                "entreprise": request.company,
                "poste": request.job_title,
                "ton": request.tone.as_deref().unwrap_or("formal"),
                "longueur": request.length.as_deref().unwrap_or("medium"),
                "lettre_precedente": request.previous_cover_letter,
                "instruction": request.instruction,
            })
        } else {
            serde_json::json!({
                "catalogue": compact_catalog,
                "entreprise": request.company,
                "poste": request.job_title,
                "ton": request.tone.as_deref().unwrap_or("formal"),
                "longueur": request.length.as_deref().unwrap_or("medium"),
                "contexte": request.context.as_deref().map(|value| truncate_chars(&clean_offer_context(value), 4_000)),
                "instruction": request.instruction,
            })
        }
        .to_string();
        progres(&notifier, &id, "Rédaction", None, None);
        let provider = self.provider_for(AiTask::WriteLetter).await?;
        let system = if iterating {
            COVER_LETTER_ITERATION_SYSTEM
        } else {
            COVER_LETTER_SYSTEM
        };
        let (plan, mut tokens) = cancel(
            &token,
            generate_json::<CoverLetterPlan>(
                provider.clone(),
                &bloc_donnees("brief", &context),
                system,
            ),
        )
        .await?;
        let fact_limit = fact_limit_for_request(&request)?;
        let evidence = resolve_letter_evidence(&catalog, &plan, fact_limit)?;
        let cleaned_offer = request
            .context
            .as_deref()
            .map(clean_offer_context)
            .unwrap_or_default();
        let company = request.company.as_deref().unwrap_or("").trim();
        let job_title = request.job_title.as_deref().unwrap_or("").trim();
        let draft_context = serde_json::json!({
            "evidences": evidence.iter().map(|e| serde_json::json!({
                "id": e.id,
                "kind": e.kind,
                "text": e.text,
            })).collect::<Vec<_>>(),
            "entreprise": company,
            "poste": job_title,
            "ton": request.tone.as_deref().unwrap_or("formal"),
            "longueur": request.length.as_deref().unwrap_or("medium"),
            "contexte_nettoye": truncate_chars(&cleaned_offer, 3_000),
            "lettre_precedente": request.previous_cover_letter,
            "instruction": request.instruction,
        })
        .to_string();
        progres(&notifier, &id, "Rédaction", None, tokens);
        let (draft, draft_tokens) = cancel(
            &token,
            generate_json::<CoverLetterDraft>(
                provider.clone(),
                &bloc_donnees("brief", &draft_context),
                COVER_LETTER_DRAFT_SYSTEM,
            ),
        )
        .await?;
        tokens = add_tokens(tokens, draft_tokens);
        let mut cover_letter = draft.letter.trim().to_owned();
        if cover_letter.chars().count() < 80 {
            // Fallback dégradé si le modèle n'a pas produit de prose.
            cover_letter = render_grounded_letter(&catalog, &plan, &request)?;
        } else {
            cover_letter =
                ground_cover_letter(&cover_letter, &evidence, company, job_title, &cleaned_offer);
        }
        progres(&notifier, &id, "Relecture du français", None, tokens);
        let correction_request = LanguageCorrectionRequest {
            generation_id: id.clone(),
            fields: vec![LanguageCorrectionField {
                id: "letter:body".into(),
                text: cover_letter.clone(),
            }],
        };
        let (correction, call_tokens) = cancel(
            &token,
            correct_language_fields(provider, &correction_request),
        )
        .await?;
        tokens = add_tokens(tokens, call_tokens);
        if let Some(field) = correction.fields.first() {
            cover_letter = field.text.clone();
        }
        let fragments = decouper_fragments(&cover_letter);
        for chunk in &fragments {
            if token.is_cancelled() {
                return Err(AppError::Cancelled);
            }
            progres(&notifier, &id, "Rédaction", Some(chunk.clone()), tokens);
        }
        Ok(execution(started_at, cover_letter, tokens))
    }

    /// Relit les champs textuels du document courant sans modifier sa structure.
    pub async fn correct_french(
        &self,
        request: LanguageCorrectionRequest,
        notifier: impl Fn(AiProgress),
    ) -> AppResult<AiExecution<LanguageCorrectionResult>> {
        let started_at = std::time::Instant::now();
        validate_language_correction_request(&request)?;
        let id = request.generation_id.clone();
        let token = self.start(&id);
        let _guard = GenerationEnCours {
            service: self,
            id: id.clone(),
            token: Arc::clone(&token),
        };
        progres(&notifier, &id, "Relecture du français", None, None);
        let (output, tokens) = cancel(
            &token,
            correct_language_fields(self.provider_for(AiTask::WriteLetter).await?, &request),
        )
        .await?;
        progres(&notifier, &id, "Correction terminée", None, tokens);
        Ok(execution(started_at, output, tokens))
    }
}

/// Construit un socle factuel sans fournisseur : aucune sélection sémantique n'est simulée,
/// mais toutes les données du profil restent accessibles dans Suggestions.
pub(super) fn fallback_resume_generation(
    profile: &Profile,
    raw_offer: &str,
    reason: String,
) -> ResumeGeneration {
    let title = raw_offer
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("Offre ciblée")
        .chars()
        .take(160)
        .collect::<String>();
    let job_offer = StructuredListing {
        title,
        ..StructuredListing::default()
    };
    let resume_text = profile
        .identity
        .resume
        .as_deref()
        .or(profile.identity.title.as_deref())
        .or_else(|| {
            profile
                .experiences
                .first()
                .map(|experience| experience.title.as_str())
        })
        .unwrap_or("Profil à compléter.")
        .to_owned();
    let resume = GeneratedResume {
        resume: resume_text,
        experiences: profile
            .experiences
            .iter()
            .map(|experience| GeneratedExperience {
                title: experience.title.clone(),
                company: experience.company.clone(),
                description: experience.description.clone().unwrap_or_default(),
            })
            .collect(),
        skills: Vec::new(),
        education: profile
            .education
            .iter()
            .map(|education| GeneratedEducation {
                degree: education.degree.clone(),
                school: education.school.clone(),
            })
            .collect(),
    };
    ResumeGeneration {
        resume,
        analysis: AtsAnalysis::default(),
        profile_score: profile_score(profile, &job_offer),
        job_offer,
        recommendation_error: Some(reason),
    }
}

pub(super) fn resume_correction_request(
    generation_id: &str,
    resume: &GeneratedResume,
) -> LanguageCorrectionRequest {
    let mut fields = Vec::with_capacity(resume.experiences.len() + 1);
    if !resume.resume.trim().is_empty() {
        fields.push(LanguageCorrectionField {
            id: "resume:profile".into(),
            text: resume.resume.clone(),
        });
    }
    fields.extend(
        resume
            .experiences
            .iter()
            .enumerate()
            .filter(|(_, experience)| !experience.description.trim().is_empty())
            .map(|(index, experience)| LanguageCorrectionField {
                id: format!("resume:experience:{index}"),
                text: experience.description.clone(),
            }),
    );
    LanguageCorrectionRequest {
        generation_id: generation_id.into(),
        fields,
    }
}

pub(super) fn apply_resume_correction(
    resume: &mut GeneratedResume,
    correction: &LanguageCorrectionResult,
) {
    for field in &correction.fields {
        if field.id == "resume:profile" {
            resume.resume.clone_from(&field.text);
        } else if let Some(index) = field
            .id
            .strip_prefix("resume:experience:")
            .and_then(|value| value.parse::<usize>().ok())
        {
            if let Some(experience) = resume.experiences.get_mut(index) {
                experience.description.clone_from(&field.text);
            }
        }
    }
}

pub(super) async fn correct_language_fields(
    provider: Arc<dyn LlmGenerator>,
    request: &LanguageCorrectionRequest,
) -> AppResult<(LanguageCorrectionResult, Option<u32>)> {
    validate_language_correction_request(request)?;
    let source = serde_json::to_string(&serde_json::json!({ "fields": request.fields }))
        .map_err(|error| AppError::Serialization(error.to_string()))?;
    let (candidate, tokens): (LanguageCorrectionResult, Option<u32>) = generate_json(
        provider,
        &bloc_donnees("document", &source),
        FRENCH_CORRECTION_SYSTEM,
    )
    .await?;
    Ok((ground_language_correction(request, candidate), tokens))
}

/// Réordonne la réponse sur la source et refuse localement toute correction suspecte.
/// Les identifiants inventés ou dupliqués ne peuvent donc jamais atteindre le document.
pub(super) fn ground_language_correction(
    request: &LanguageCorrectionRequest,
    candidate: LanguageCorrectionResult,
) -> LanguageCorrectionResult {
    use std::collections::HashMap;

    let mut by_id = HashMap::new();
    for field in candidate.fields {
        if by_id.contains_key(field.id.as_str()) {
            continue;
        }
        by_id.insert(field.id.clone(), field.text);
    }
    LanguageCorrectionResult {
        fields: request
            .fields
            .iter()
            .map(|source| {
                let corrected = by_id
                    .get(source.id.as_str())
                    .filter(|value| conservative_correction(&source.text, value))
                    .cloned()
                    .unwrap_or_else(|| source.text.clone());
                LanguageCorrectionField {
                    id: source.id.clone(),
                    text: corrected,
                }
            })
            .collect(),
    }
}

pub(super) fn conservative_correction(source: &str, corrected: &str) -> bool {
    if corrected.trim().is_empty() {
        return false;
    }
    let source_len = source.chars().count();
    let corrected_len = corrected.chars().count();
    if source_len >= 20
        && (corrected_len.saturating_mul(10) < source_len.saturating_mul(6)
            || corrected_len > source_len.saturating_mul(14) / 10)
    {
        return false;
    }
    let corrected_protected = protected_fragments(corrected);
    protected_fragments(source)
        .into_iter()
        .all(|fragment| corrected_protected.contains(&fragment))
        && trigram_similarity(source, corrected) >= 0.62
}

/// Une relecture légitime conserve l'essentiel des séquences de caractères. Cette mesure
/// linéaire rejette une réécriture de longueur comparable qui garderait seulement les noms
/// propres et les chiffres protégés, sans prétendre juger elle-même la qualité du français.
pub(super) fn trigram_similarity(source: &str, corrected: &str) -> f32 {
    use std::collections::HashMap;

    pub(super) fn trigrams(value: &str) -> HashMap<[char; 3], usize> {
        let normalized: Vec<_> = value
            .to_lowercase()
            .chars()
            .filter(|character| character.is_alphanumeric() || character.is_whitespace())
            .collect();
        let mut counts = HashMap::new();
        for window in normalized.windows(3) {
            *counts.entry([window[0], window[1], window[2]]).or_insert(0) += 1;
        }
        counts
    }

    let source = trigrams(source);
    let corrected = trigrams(corrected);
    let source_count: usize = source.values().sum();
    let corrected_count: usize = corrected.values().sum();
    let denominator = source_count.max(corrected_count);
    if denominator == 0 {
        return 1.0;
    }
    let common: usize = source
        .iter()
        .map(|(trigram, count)| count.min(corrected.get(trigram).unwrap_or(&0)))
        .sum();
    common as f32 / denominator as f32
}

/// Chiffres, coordonnées et noms de technologies à casse distinctive sont des faits, pas de
/// la prose. Une correction qui en perd ou en altère un est rejetée en bloc.
pub(super) fn protected_fragments(value: &str) -> std::collections::HashSet<String> {
    value
        .split_whitespace()
        .map(|token| {
            token.trim_matches(|character: char| {
                matches!(
                    character,
                    '.' | ',' | ';' | ':' | '!' | '?' | '(' | ')' | '[' | ']' | '{' | '}'
                )
            })
        })
        .filter(|token| {
            token.chars().any(|character| character.is_ascii_digit())
                || token.contains('@')
                || token.contains("://")
                || token.chars().any(char::is_uppercase)
                || (token.chars().count() > 1
                    && token
                        .chars()
                        .filter(|character| character.is_alphabetic())
                        .all(char::is_uppercase))
        })
        .map(str::to_owned)
        .collect()
}
