//! Orchestration native des cas d'usage IA, sans commande IPC.

use crate::modules::ia::cache::{cache_key, CacheEntry, CacheIaRepository};
use crate::modules::ia::cv_engine::CvEngine;
use crate::modules::ia::cv_model::{
    AtsAnalysis, CvGeneration, GeneratedCv, LetterGenerationRequest, MatchScore, OfferAnalysis,
    ParsedOffer,
};
use crate::modules::ia::factory::build_provider_pinned;
use crate::modules::ia::profile_extraction::profile_is_empty;
use crate::modules::ia::scoring;
use crate::shared::error::{AppError, AppResult};
use crate::shared::llm::{validate_llm_endpoint, AnalysisMode, LlmConfig, ProviderKind};
use crate::shared::state::{
    AppState, AppelLlm, MetriquesRepository, OperationLlm, OrigineScore, ScoreAts,
};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::Path;
use tokio_util::sync::CancellationToken;

const MAX_PDF_BYTES: usize = 10 * 1024 * 1024;
const MAX_TEXT_CHARS: usize = 200_000;
const MAX_EXTRACTED_CV_CHARS: usize = 1_000_000;

/// Résultat complet d'une analyse de CV importé.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportedCvAnalysis {
    /// CV structuré depuis le PDF.
    pub cv: GeneratedCv,
    /// Offre structurée.
    pub offer: ParsedOffer,
    /// Score local déterministe.
    pub score: MatchScore,
    /// Recommandations ATS.
    pub analysis: AtsAnalysis,
}

/// Analyse une offre et calcule son score contre le profil local.
///
/// # Errors
/// Retourne une erreur de validation, de configuration ou de provider.
pub async fn analyze_offer(state: &AppState, raw_text: String) -> AppResult<OfferAnalysis> {
    validate_text(&raw_text, "L'offre")?;
    let (settings, pin) = configured_settings(state).await?;
    let profile = state.profil.get()?;
    let engine = CvEngine::with_mode(
        build_provider_pinned(&settings.llm, pin),
        settings.llm.resolved_mode(),
    );
    let parsed = cached(
        state,
        OperationLlm::ParseOffer,
        &settings.llm,
        &raw_text,
        engine.parse_offer(&raw_text),
    )
    .await?;
    let score = scoring::score(&profile, &parsed);
    Ok(OfferAnalysis { parsed, score })
}

/// Génère un CV, puis produit son analyse ATS.
///
/// # Errors
/// Retourne l'erreur métier ou `Cancelled` si le jeton est annulé.
pub async fn generate_cv(
    state: &AppState,
    offer: ParsedOffer,
    score: MatchScore,
    cancellation: CancellationToken,
) -> AppResult<CvGeneration> {
    let (settings, pin) = configured_settings(state).await?;
    let profile = state.profil.get()?;
    if profile_is_empty(&profile) {
        return Err(AppError::Validation(
            "Complétez votre profil avant de générer un CV.".into(),
        ));
    }
    let engine = CvEngine::with_mode(
        build_provider_pinned(&settings.llm, pin),
        settings.llm.resolved_mode(),
    );
    let generation = measured(
        state,
        OperationLlm::GenerateCv,
        &settings.llm,
        engine.generate_cv(&profile, &offer, &score),
    );
    // L'annulation couvre **toute** la séquence, analyse ATS comprise. Le `select!` ne
    // protégeait auparavant que la génération : un clic sur « Annuler » pendant la seconde
    // moitié n'interrompait rien, l'interface affichait « Annulation demandée… » et
    // l'opération se poursuivait jusqu'à son terme ou jusqu'aux 30 minutes de
    // `PROVIDER_GENERATION_TIMEOUT`, bouton bloqué et chronomètre tournant.
    let travail = async {
        let cv = generation.await?;
        let cache_input = serde_json::json!({ "cv": cv, "offer": offer }).to_string();
        let analysis = cached(
            state,
            OperationLlm::AnalyzeAts,
            &settings.llm,
            &cache_input,
            engine.analyze_ats(&cv, &offer),
        )
        .await?;
        if let Err(error) = state.metriques.enregistrer_score(&ScoreAts {
            score: analysis.score,
            origine: OrigineScore::Genere,
            cree_le: now(),
        }) {
            tracing::warn!(erreur = %error, "score ATS non enregistré");
        }
        Ok(CvGeneration { cv, analysis })
    };
    tokio::select! {
        resultat = travail => resultat,
        () = cancellation.cancelled() => {
            tracing::info!("génération de CV annulée par l'utilisateur");
            Err(AppError::Cancelled)
        }
    }
}

/// Analyse un PDF existant contre une offre, sans modifier le fichier.
///
/// # Errors
/// Retourne une erreur si le PDF, l'offre ou le provider sont invalides.
pub async fn analyze_imported_cv(
    state: &AppState,
    pdf_path: &Path,
    offer_text: String,
    cancellation: CancellationToken,
) -> AppResult<ImportedCvAnalysis> {
    validate_text(&offer_text, "L'offre")?;
    let extracted = lire_et_extraire_pdf(pdf_path).await?;
    if extracted.chars().count() > MAX_EXTRACTED_CV_CHARS {
        return Err(AppError::Validation(
            "Le texte extrait du PDF dépasse la limite autorisée.".into(),
        ));
    }
    let (settings, pin) = configured_settings(state).await?;
    let engine = CvEngine::with_mode(
        build_provider_pinned(&settings.llm, pin),
        settings.llm.resolved_mode(),
    );
    let work = async {
        let cv = cached(
            state,
            OperationLlm::ParseCv,
            &settings.llm,
            &extracted,
            engine.parse_cv(&extracted),
        )
        .await?;
        let offer = cached(
            state,
            OperationLlm::ParseOffer,
            &settings.llm,
            &offer_text,
            engine.parse_offer(&offer_text),
        )
        .await?;
        let score = scoring::score_imported(&cv, &offer);
        let cache_input = serde_json::json!({ "cv": cv, "offer": offer }).to_string();
        let analysis = cached(
            state,
            OperationLlm::AnalyzeAts,
            &settings.llm,
            &cache_input,
            engine.analyze_ats(&cv, &offer),
        )
        .await?;
        let _ = state.metriques.enregistrer_score(&ScoreAts {
            score: analysis.score,
            origine: OrigineScore::Importe,
            cree_le: now(),
        });
        Ok(ImportedCvAnalysis {
            cv,
            offer,
            score,
            analysis,
        })
    };
    tokio::select! {
        result = work => result,
        () = cancellation.cancelled() => Err(AppError::Cancelled),
    }
}

/// Extrait un profil structuré depuis un PDF sans le persister.
///
/// # Errors
/// Retourne une erreur si le PDF est invalide, le provider indisponible ou le profil vide.
pub async fn extract_profile_from_pdf(
    state: &AppState,
    pdf_path: &Path,
    cancellation: CancellationToken,
) -> AppResult<crate::shared::profile::Profile> {
    let extracted = lire_et_extraire_pdf(pdf_path).await?;
    if extracted.chars().count() > MAX_EXTRACTED_CV_CHARS {
        return Err(AppError::Validation(
            "Le texte extrait du PDF dépasse la limite autorisée.".into(),
        ));
    }
    let cleaned = crate::shared::pdf::clean_cv_text(&extracted);
    let (settings, pin) = configured_settings(state).await?;
    let config = LlmConfig {
        temperature: 0.0,
        ..settings.llm
    };
    let engine = CvEngine::with_mode(build_provider_pinned(&config, pin), config.resolved_mode());
    let extraction = cached(
        state,
        OperationLlm::ParseCv,
        &config,
        &cleaned,
        engine.extract_profile(&cleaned),
    );
    let profile = tokio::select! {
        result = extraction => result?,
        () = cancellation.cancelled() => return Err(AppError::Cancelled),
    };
    if profile_is_empty(&profile) {
        return Err(AppError::Validation(
            "Aucune information exploitable n'a été détectée dans ce CV.".into(),
        ));
    }
    Ok(profile)
}

/// Génère une lettre en livrant chaque fragment au callback.
///
/// # Errors
/// Retourne une erreur de profil, de provider ou d'annulation.
pub async fn generate_cover_letter(
    state: &AppState,
    request: &LetterGenerationRequest,
    cancellation: CancellationToken,
    on_chunk: &mut (dyn FnMut(String) + Send),
) -> AppResult<String> {
    validate_letter_request(request)?;
    let (settings, pin) = configured_settings(state).await?;
    let profile = request
        .profile
        .as_ref()
        .filter(|profile| !profile_is_empty(profile))
        .cloned()
        .unwrap_or(state.profil.get()?);
    if profile_is_empty(&profile) {
        return Err(AppError::Validation(
            "Complétez votre profil avant de générer une lettre.".into(),
        ));
    }
    let engine = CvEngine::with_mode(
        build_provider_pinned(&settings.llm, pin),
        settings.llm.resolved_mode(),
    );
    let stream = measured(
        state,
        OperationLlm::CoverLetter,
        &settings.llm,
        engine.stream_cover_letter(&profile, request, on_chunk),
    );
    tokio::select! {
        result = stream => result,
        () = cancellation.cancelled() => Err(AppError::Cancelled),
    }
}

/// Analyse et persiste le compte rendu d'un entretien.
///
/// # Errors
/// Retourne une erreur si le compte rendu est vide, le provider échoue ou l'appel est annulé.
pub async fn analyze_interview(
    state: &AppState,
    interview_id: uuid::Uuid,
    cancellation: CancellationToken,
) -> AppResult<crate::shared::types::AnalyseEntretien> {
    let interview = state.entretiens.obtenir(interview_id)?;
    let report = interview.compte_rendu.ok_or_else(|| {
        AppError::Validation("Rédigez un compte rendu avant de lancer l'analyse.".into())
    })?;
    if report.trim().is_empty() {
        return Err(AppError::Validation(
            "Rédigez un compte rendu avant de lancer l'analyse.".into(),
        ));
    }
    let (settings, pin) = configured_settings(state).await?;
    let engine = CvEngine::with_mode(
        build_provider_pinned(&settings.llm, pin),
        settings.llm.resolved_mode(),
    );
    let analysis = tokio::select! {
        result = cached(
            state,
            OperationLlm::AnalyserEntretien,
            &settings.llm,
            &report,
            engine.analyser_entretien(&report),
        ) => result?,
        () = cancellation.cancelled() => return Err(AppError::Cancelled),
    };
    state
        .entretiens
        .enregistrer_analyse(interview_id, &analysis)?;
    Ok(analysis)
}

/// Vérifie la configuration IA et renvoie, avec elle, l'adresse validée de l'endpoint.
///
/// L'adresse est épinglée sur le client HTTP du fournisseur : la connexion emprunte exactement
/// celle qui a passé le contrôle anti-SSRF, au lieu d'être re-résolue au moment de la requête.
async fn configured_settings(
    state: &AppState,
) -> AppResult<(
    crate::shared::state::AppSettings,
    Option<crate::shared::llm::EndpointPin>,
)> {
    let settings = state.secure_settings()?;
    if !settings.llm.est_configure() {
        return Err(AppError::Validation(
            "IA non configurée : ouvrez les Paramètres pour choisir un fournisseur et un modèle."
                .into(),
        ));
    }
    let pin = validate_llm_endpoint(&settings.llm).await?;
    Ok((settings, pin))
}

async fn measured<T>(
    state: &AppState,
    operation: OperationLlm,
    config: &LlmConfig,
    future: impl std::future::Future<Output = AppResult<T>>,
) -> AppResult<T> {
    let started = std::time::Instant::now();
    let result = future.await;
    let elapsed = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    // Le contenu de l'appel n'est jamais journalisé : il porte le profil et les offres de
    // l'utilisateur. Seuls l'opération, le fournisseur, la latence et l'issue le sont.
    tracing::info!(
        operation = operation.as_str(),
        provider = provider_name(&config.provider),
        modele = %config.model,
        latence_ms = elapsed,
        succes = result.is_ok(),
        "appel IA"
    );
    if let Err(error) = state.metriques.enregistrer_appel(&AppelLlm {
        operation,
        provider: provider_name(&config.provider).into(),
        modele: config.model.clone(),
        latence_ms: elapsed,
        succes: result.is_ok(),
        cree_le: now(),
    }) {
        tracing::warn!(erreur = %error, "télémétrie IA non enregistrée");
    }
    result
}

async fn cached<T>(
    state: &AppState,
    operation: OperationLlm,
    config: &LlmConfig,
    input: &str,
    future: impl std::future::Future<Output = AppResult<T>>,
) -> AppResult<T>
where
    T: Serialize + serde::de::DeserializeOwned,
{
    let provider = provider_name(&config.provider);
    let key = cache_key(
        provider,
        &config.model,
        mode_name(config.resolved_mode()),
        operation.as_str(),
        input,
    );
    match state.cache_ia.get(&key) {
        Ok(Some(raw)) => match serde_json::from_str(&raw) {
            Ok(value) => {
                tracing::debug!(operation = operation.as_str(), "cache IA : succès");
                return Ok(value);
            }
            // Entrée illisible : on la traite comme absente et on refait l'appel.
            Err(error) => tracing::warn!(erreur = %error, "entrée de cache IA illisible"),
        },
        Ok(None) => {}
        Err(error) => tracing::warn!(erreur = %error, "cache IA inaccessible en lecture"),
    }
    let value = measured(state, operation, config, future).await?;
    if let Ok(raw) = serde_json::to_string(&value) {
        if let Err(error) = state.cache_ia.put(&CacheEntry {
            cle: key,
            valeur: raw,
            provider: provider.into(),
            modele: config.model.clone(),
            operation: operation.as_str().into(),
            cree_le: now(),
        }) {
            tracing::warn!(erreur = %error, "mise en cache IA impossible");
        }
    }
    Ok(value)
}

fn validate_text(value: &str, label: &str) -> AppResult<()> {
    let count = value.chars().count();
    if count == 0 || count > MAX_TEXT_CHARS {
        return Err(AppError::Validation(format!(
            "{label} doit contenir entre 1 et 200 000 caractères."
        )));
    }
    Ok(())
}

fn validate_letter_request(request: &LetterGenerationRequest) -> AppResult<()> {
    if request.generation_id.trim().is_empty() {
        return Err(AppError::Validation(
            "Identifiant de génération manquant.".into(),
        ));
    }
    if request
        .tone
        .as_deref()
        .is_some_and(|tone| !matches!(tone, "formal" | "casual" | "creative"))
    {
        return Err(AppError::Validation("Ton de lettre invalide.".into()));
    }
    Ok(())
}

/// Lit un PDF et en extrait le texte **hors du fil asynchrone**.
///
/// La lecture (jusqu'à 10 Mo par `std::fs`) et surtout l'extraction (`pdf::extract_text`, un
/// traitement CPU non borné) immobilisaient un fil de travail du runtime Tokio pour toute leur
/// durée, réduisant d'autant le parallélisme disponible aux autres tâches — téléchargements,
/// appels IA concurrents.
async fn lire_et_extraire_pdf(path: &Path) -> AppResult<String> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let bytes = read_pdf(&path)?;
        crate::shared::pdf::extract_text(&bytes)
    })
    .await
    .map_err(|error| AppError::Validation(format!("Lecture du PDF interrompue : {error}")))?
}

fn read_pdf(path: &Path) -> AppResult<Vec<u8>> {
    if !path.is_file()
        || path
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .is_none_or(|extension| !extension.eq_ignore_ascii_case("pdf"))
    {
        return Err(AppError::Validation(
            "Fichier PDF introuvable ou invalide.".into(),
        ));
    }
    let metadata = std::fs::metadata(path)
        .map_err(|error| AppError::Validation(format!("Impossible de lire le PDF : {error}")))?;
    if metadata.len() > MAX_PDF_BYTES as u64 {
        return Err(AppError::Validation("Le PDF dépasse 10 Mo.".into()));
    }
    let file = std::fs::File::open(path)
        .map_err(|error| AppError::Validation(format!("Impossible d'ouvrir le PDF : {error}")))?;
    let mut bytes = Vec::new();
    file.take((MAX_PDF_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| AppError::Validation(format!("Impossible de lire le PDF : {error}")))?;
    if !bytes.starts_with(b"%PDF-") {
        return Err(AppError::Validation(
            "Le fichier n'est pas un PDF valide.".into(),
        ));
    }
    Ok(bytes)
}

const fn provider_name(provider: &ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Ollama => "ollama",
        ProviderKind::Claude => "claude",
        ProviderKind::OpenAI => "openai",
        ProviderKind::Gemini => "gemini",
        ProviderKind::Mistral => "mistral",
        ProviderKind::Nvidia => "nvidia",
        ProviderKind::Custom(_) => "custom",
    }
}

const fn mode_name(mode: AnalysisMode) -> &'static str {
    match mode {
        AnalysisMode::Auto => "auto",
        AnalysisMode::Small => "small",
        AnalysisMode::Standard => "standard",
        AnalysisMode::Advanced => "advanced",
    }
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
#[path = "tests/service/mod.rs"]
mod tests;
