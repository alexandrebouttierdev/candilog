//! Génération de documents et analyse de CV avec progression et annulation.

use super::ManagedOllamaService;
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::ai::domain::*;
#[cfg(test)]
use crate::features::ai::infrastructure::GenerationOutput;
use crate::features::ai::infrastructure::{benchmark_pdf_path, load_config, load_task_config};
use crate::features::ai::infrastructure::{
    build_provider, extract_pdf, render_pdf_pages, try_extract_pdf_text, LlmGenerator, VisionImage,
};
use crate::features::profile::domain::{build_preview, Profile, ProfileRepository};
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

mod benchmark;
mod documents;
mod profile_import;
mod prompts;
mod resume_analysis;

#[cfg(test)]
use benchmark::*;
#[cfg(test)]
use documents::*;
use profile_import::*;
use prompts::*;

pub struct AiService {
    pool: SqlitePool,
    /// Profil lu par son contrat : la feature IA ne connaît pas son stockage.
    profiles: Arc<dyn ProfileRepository>,
    managed_ollama: Arc<ManagedOllamaService>,
    generations: Mutex<HashMap<String, Arc<CancellationToken>>>,
    /// Dernier CV choisi dans le dialogue natif, jamais exposé à l'IPC.
    ///
    /// L'écran « Analyser » sépare le choix du fichier de son analyse : le chemin devait donc
    /// survivre entre deux commandes. Le faire transiter par le frontend en aurait fait une
    /// entrée non fiable, capable de désigner n'importe quel PDF du disque
    /// (`docs/CODE_RULES.md` §10).
    selected_resume: Mutex<Option<PathBuf>>,
}

impl AiService {
    #[must_use]
    pub fn new(
        pool: SqlitePool,
        profiles: Arc<dyn ProfileRepository>,
        managed_ollama: Arc<ManagedOllamaService>,
    ) -> Self {
        Self {
            pool,
            profiles,
            managed_ollama,
            generations: Mutex::new(HashMap::new()),
            selected_resume: Mutex::new(None),
        }
    }

    /// Retient le CV que l'utilisateur vient de désigner dans le dialogue natif.
    pub fn remember_selected_resume(&self, path: PathBuf) {
        *self
            .selected_resume
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(path);
    }

    /// Chemin du CV à analyser.
    ///
    /// # Errors
    /// Retourne `Validation` si aucun fichier n'a été choisi : l'analyse ne lit que ce que
    /// l'utilisateur a explicitement sélectionné.
    fn selected_resume_path(&self) -> AppResult<PathBuf> {
        self.selected_resume
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .ok_or_else(|| {
                AppError::Validation(
                    "Choisissez le CV PDF à analyser avant de lancer l'analyse.".into(),
                )
            })
    }

    pub fn cancel(&self, id: &str) {
        if let Some(token) = self
            .generations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(id)
        {
            token.cancel();
        }
    }

    fn start(&self, id: &str) -> Arc<CancellationToken> {
        let token = Arc::new(CancellationToken::new());
        let mut generations = self
            .generations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(ancien) = generations.insert(id.to_owned(), token.clone()) {
            ancien.cancel();
        }
        token
    }

    fn finish(&self, id: &str, token: &Arc<CancellationToken>) {
        let mut generations = self
            .generations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if generations
            .get(id)
            .is_some_and(|current| Arc::ptr_eq(current, token))
        {
            generations.remove(id);
        }
    }

    fn profile(&self) -> AppResult<Profile> {
        Ok(self.profiles.get()?.0)
    }

    async fn provider(&self) -> AppResult<Arc<dyn LlmGenerator>> {
        self.build(load_config(&self.pool)?, false).await
    }

    /// Fournisseur de la tâche : sa route si elle en a une, le fournisseur principal sinon.
    async fn provider_for(&self, task: AiTask) -> AppResult<Arc<dyn LlmGenerator>> {
        let (config, routed) = load_task_config(&self.pool, task)?;
        self.build(config, routed).await
    }

    async fn build(&self, config: LlmConfig, routed: bool) -> AppResult<Arc<dyn LlmGenerator>> {
        if matches!(config.provider, ProviderKind::CandilogLocal) {
            let base_url = self.managed_ollama.ensure_runtime_ready().await?;
            if routed {
                // Une route locale désigne un modèle installé précis ; s'il a été désinstallé,
                // la tâche s'arrête au lieu de tourner sur un autre modèle.
                let installed = self
                    .managed_ollama
                    .status()?
                    .models
                    .into_iter()
                    .any(|model| model.installed && model.definition.ollama_tag == config.model);
                if !installed {
                    return Err(AppError::Provider(format!(
                        "Le modèle local « {} » n'est plus installé. Réinstallez-le ou choisissez un autre modèle dans Intelligence artificielle.",
                        config.model
                    )));
                }
                return build_provider(&LlmConfig {
                    provider: ProviderKind::Ollama,
                    api_key: None,
                    endpoint: Some(base_url),
                    model: config.model,
                    temperature: config.temperature,
                    mode: config.mode,
                })
                .await;
            }
            let model = self
                .managed_ollama
                .active_ollama_tag()?
                .filter(|tag| !tag.trim().is_empty())
                .or_else(|| {
                    if config.model.trim().is_empty() {
                        None
                    } else {
                        Some(config.model.clone())
                    }
                })
                .ok_or_else(|| {
                    AppError::Provider(
                        "Choisissez et installez un modèle local avant d'utiliser l'IA.".into(),
                    )
                })?;
            let ollama_config = LlmConfig {
                provider: ProviderKind::Ollama,
                api_key: None,
                endpoint: Some(base_url),
                model,
                temperature: config.temperature,
                mode: config.mode,
            };
            build_provider(&ollama_config).await
        } else {
            build_provider(&config).await
        }
    }
}

fn provider_label(config: &LlmConfig) -> String {
    match config.provider {
        ProviderKind::CandilogLocal => "IA locale Candilog".into(),
        ProviderKind::Ollama => "Ollama".into(),
        ProviderKind::Claude => "Claude".into(),
        ProviderKind::OpenAI => "OpenAI".into(),
        ProviderKind::Gemini => "Gemini".into(),
        ProviderKind::Mistral => "Mistral".into(),
        ProviderKind::DeepSeek => "DeepSeek".into(),
        ProviderKind::Custom(_) => "API personnalisée".into(),
    }
}

struct GenerationEnCours<'a> {
    service: &'a AiService,
    id: String,
    token: Arc<CancellationToken>,
}

impl Drop for GenerationEnCours<'_> {
    fn drop(&mut self) {
        self.service.finish(&self.id, &self.token);
    }
}

fn execution<T>(
    started_at: std::time::Instant,
    output: T,
    tokens_used: Option<u32>,
) -> AiExecution<T> {
    AiExecution {
        output,
        elapsed_ms: started_at.elapsed().as_millis().min(u128::from(u32::MAX)) as u32,
        tokens_used,
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let count = value.chars().count();
    if count <= max_chars {
        return value.to_owned();
    }
    let mut truncated: String = value.chars().take(max_chars.saturating_sub(1)).collect();
    truncated.push('…');
    truncated
}

async fn generate_json<T: serde::de::DeserializeOwned + ValidateAiOutput>(
    provider: Arc<dyn LlmGenerator>,
    prompt: &str,
    system: &str,
) -> AppResult<(T, Option<u32>)> {
    let mut current = prompt.to_owned();
    let mut derniere = None;
    // La reprise sur JSON invalide consomme un second appel réel : ses tokens doivent
    // s'ajouter à ceux du premier, pas les remplacer.
    let mut tokens = Some(0_u32);
    for _ in 0..2 {
        let sortie = provider.generate(&current, system, true).await?;
        tokens = add_tokens(tokens, sortie.tokens);
        validate_raw_output(&sortie.text)?;
        match parse_json::<T>(&sortie.text) {
            Ok(value) => {
                value.validate_ai_output()?;
                return Ok((value, tokens));
            }
            Err(error) => {
                derniere = Some(error.to_string());
                current = format!(
                    "{prompt}\n\nLa réponse précédente était un JSON invalide. Renvoie l'objet complet, sans Markdown. N'inclus pas la réponse précédente."
                );
            }
        }
    }
    Err(AppError::Serialization(
        derniere.unwrap_or_else(|| "Réponse IA illisible".into()),
    ))
}

async fn generate_json_vision<T: serde::de::DeserializeOwned + ValidateAiOutput>(
    provider: Arc<dyn LlmGenerator>,
    prompt: &str,
    system: &str,
    images: &[VisionImage],
) -> AppResult<(T, Option<u32>)> {
    let mut current = prompt.to_owned();
    let mut derniere = None;
    let mut tokens = Some(0_u32);
    for _ in 0..2 {
        let sortie = provider
            .generate_vision(&current, system, images, true)
            .await?;
        tokens = add_tokens(tokens, sortie.tokens);
        validate_raw_output(&sortie.text)?;
        match parse_json::<T>(&sortie.text) {
            Ok(value) => {
                value.validate_ai_output()?;
                return Ok((value, tokens));
            }
            Err(error) => {
                derniere = Some(error.to_string());
                current = format!(
                    "{prompt}\n\nLa réponse précédente était un JSON invalide. Renvoie l'objet complet, sans Markdown. N'inclus pas la réponse précédente."
                );
            }
        }
    }
    Err(AppError::Serialization(
        derniere.unwrap_or_else(|| "Réponse IA illisible".into()),
    ))
}

fn add_tokens(total: Option<u32>, call: Option<u32>) -> Option<u32> {
    Some(total?.saturating_add(call?))
}

fn parse_json<T: serde::de::DeserializeOwned>(raw: &str) -> Result<T, serde_json::Error> {
    let extracted = match (raw.find('{'), raw.rfind('}')) {
        (Some(a), Some(b)) if b >= a => &raw[a..=b],
        _ => raw,
    };
    serde_json::from_str(extracted).or_else(|strict| {
        jsonrepair_rs::jsonrepair(extracted)
            .ok()
            .and_then(|r| serde_json::from_str(&r).ok())
            .ok_or(strict)
    })
}

async fn cancel<T>(
    token: &CancellationToken,
    work: impl Future<Output = AppResult<T>>,
) -> AppResult<T> {
    tokio::select! { result = work => result, () = token.cancelled() => Err(AppError::Cancelled) }
}

/// Comme [`cancel`], mais réveille l'appelant chaque seconde pour publier l'avancement.
///
/// Une analyse de CV en IA locale dure une douzaine de minutes sur un portable. Sans
/// battement, l'écran reste figé sur « Analyse du CV… » et rien ne distingue une génération
/// lente d'un blocage.
async fn cancel_avec_progression<T>(
    token: &CancellationToken,
    work: impl Future<Output = AppResult<T>>,
    mut battement: impl FnMut(),
) -> AppResult<T> {
    tokio::pin!(work);
    let mut horloge = tokio::time::interval(std::time::Duration::from_secs(1));
    horloge.tick().await; // le premier top est immédiat : on l'absorbe.
    loop {
        tokio::select! {
            result = &mut work => return result,
            () = token.cancelled() => return Err(AppError::Cancelled),
            _ = horloge.tick() => battement(),
        }
    }
}
/// Encadre un contenu externe dans un bloc que le modèle doit lire comme de la donnée.
///
/// La balise porte un identifiant tiré au sort à chaque appel, et la balise fermante est
/// neutralisée dans le contenu. Un délimiteur fixe et connu (`</offre>`) pouvait figurer
/// dans l'offre elle-même : le bloc se refermait, et la suite du texte se présentait au
/// modèle au même rang que les instructions système (`docs/CODE_RULES.md` §12).
fn bloc_donnees(label: &str, contenu: &str) -> String {
    let marque = uuid::Uuid::new_v4().simple().to_string();
    let ouverture = format!("<{label} id=\"{marque}\">");
    let fermeture = format!("</{label}>");
    // Le contenu ne peut plus refermer ni le bloc générique, ni celui de cet appel.
    let contenu = contenu.replace(&fermeture, &format!("<{label}_echappe/>"));
    format!("{DONNEES_NON_FIABLES}\n{ouverture}\n{contenu}\n{fermeture}")
}
fn progres(
    notifier: &impl Fn(AiProgress),
    id: &str,
    step: &str,
    chunk: Option<String>,
    tokens_used: Option<u32>,
) {
    notifier(AiProgress {
        generation_id: id.into(),
        step: step.into(),
        chunk,
        tokens_used,
    });
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
