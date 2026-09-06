//! Runtime llama.cpp embarqué, chargé paresseusement et sérialisé par processus.

use crate::core::errors::{AppError, AppResult};
use crate::features::ai::domain::{LocalAiBackend, LocalModelId, LOCAL_AI_CONTEXT_SIZE};
use encoding_rs::UTF_8;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

const MAX_OUTPUT_TOKENS: usize = 4_096;
/// Taille de lot d'inférence : bien inférieure au contexte pour limiter la RAM allouée.
const LOCAL_AI_BATCH_SIZE: u32 = 512;
const JSON_GRAMMAR: &str = r#"
root ::= ws value ws
value ::= object | array | string | number | "true" | "false" | "null"
object ::= "{" ws (string ws ":" ws value (ws "," ws string ws ":" ws value)*)? ws "}"
array ::= "[" ws (value (ws "," ws value)*)? ws "]"
string ::= "\"" chars "\""
chars ::= ([^"\\] | "\\" (["\\/bfnrt] | "u" [0-9a-fA-F] [0-9a-fA-F] [0-9a-fA-F] [0-9a-fA-F]))*
number ::= "-"? ("0" | [1-9] [0-9]*) ("." [0-9]+)? ([eE] [+-]? [0-9]+)?
ws ::= [ \t\n\r]*
"#;

static LLAMA_BACKEND: OnceLock<Result<Arc<LlamaBackend>, String>> = OnceLock::new();

pub(super) fn shared_backend() -> AppResult<Arc<LlamaBackend>> {
    LLAMA_BACKEND
        .get_or_init(|| {
            LlamaBackend::init()
                .map(Arc::new)
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map(Arc::clone)
        .map_err(|error| {
            tracing::error!(%error, "runtime local non initialisé");
            AppError::Provider(
                "Le moteur d'IA locale n'est pas disponible sur cette machine.".into(),
            )
        })
}

struct LoadedModel {
    id: LocalModelId,
    path: PathBuf,
    model: Arc<LlamaModel>,
    backend: LocalAiBackend,
}

#[derive(Debug)]
pub struct RuntimeGeneration {
    pub text: String,
    pub prompt_tokens: u32,
    pub generated_tokens: u32,
    pub load_time_ms: u64,
    pub backend: LocalAiBackend,
}

pub struct RuntimeRequest<'a> {
    pub model_id: LocalModelId,
    pub path: &'a Path,
    pub expected_sha256: &'a str,
    pub backend: LocalAiBackend,
    pub system: &'a str,
    pub prompt: &'a str,
    pub temperature: f32,
    pub json: bool,
    pub max_output_tokens: Option<usize>,
}

#[derive(Default)]
pub struct MistralLocalRuntime {
    loaded: Mutex<Option<LoadedModel>>,
    inference: Mutex<()>,
}

impl MistralLocalRuntime {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn unload(&self) {
        self.loaded
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        tracing::info!("modèle Mistral Local libéré");
    }

    pub fn generate(&self, request: RuntimeRequest<'_>) -> AppResult<RuntimeGeneration> {
        let _inference = self
            .inference
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (model, backend, load_time_ms) = self.load(
            request.model_id,
            request.path,
            request.expected_sha256,
            request.backend,
        )?;
        let messages = [
            LlamaChatMessage::new("system".into(), request.system.into()).map_err(runtime_error)?,
            LlamaChatMessage::new("user".into(), request.prompt.into()).map_err(runtime_error)?,
        ];
        let template = model.chat_template(None).map_err(runtime_error)?;
        let formatted = model
            .apply_chat_template(&template, &messages, true)
            .map_err(runtime_error)?;
        let tokens = model
            .str_to_token(&formatted, AddBos::Always)
            .map_err(runtime_error)?;
        if tokens.is_empty() || tokens.len() >= LOCAL_AI_CONTEXT_SIZE as usize - 16 {
            return Err(AppError::Provider(
                "Le contenu dépasse la capacité du modèle local. Réduisez le texte à analyser."
                    .into(),
            ));
        }
        let remaining = LOCAL_AI_CONTEXT_SIZE as usize - tokens.len();
        let output_limit = request
            .max_output_tokens
            .unwrap_or(MAX_OUTPUT_TOKENS)
            .min(MAX_OUTPUT_TOKENS)
            .min(remaining);
        let threads = std::thread::available_parallelism()
            .map_or(4_i32, |value| {
                i32::try_from(value.get()).unwrap_or(i32::MAX)
            })
            .max(1);
        let context_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(LOCAL_AI_CONTEXT_SIZE))
            .with_n_batch(LOCAL_AI_BATCH_SIZE)
            .with_n_ubatch(512)
            .with_n_threads(threads)
            .with_n_threads_batch(threads);
        let llama_backend = shared_backend()?;
        let mut context = model
            .new_context(&llama_backend, context_params)
            .map_err(runtime_error)?;
        let mut batch = LlamaBatch::new(tokens.len().max(1), 1);
        let last = tokens.len().saturating_sub(1);
        for (index, token) in tokens.iter().copied().enumerate() {
            batch
                .add(
                    token,
                    i32::try_from(index).map_err(runtime_error)?,
                    &[0],
                    index == last,
                )
                .map_err(runtime_error)?;
        }
        context.decode(&mut batch).map_err(runtime_error)?;

        let mut samplers = Vec::with_capacity(5);
        if request.json {
            samplers
                .push(LlamaSampler::grammar(&model, JSON_GRAMMAR, "root").map_err(runtime_error)?);
        }
        samplers.push(LlamaSampler::temp(request.temperature.clamp(0.0, 2.0)));
        samplers.push(LlamaSampler::top_p(0.9, 1));
        samplers.push(LlamaSampler::min_p(0.05, 1));
        samplers.push(LlamaSampler::dist(0xC4A1_D109));
        let mut sampler = LlamaSampler::chain_simple(samplers);
        let mut decoder = UTF_8.new_decoder();
        let mut text = String::new();
        let mut generated = 0_u32;
        let mut position = i32::try_from(tokens.len()).map_err(runtime_error)?;
        for _ in 0..output_limit {
            let token = sampler.sample(&context, batch.n_tokens() - 1);
            sampler.accept(token);
            if model.is_eog_token(token) {
                break;
            }
            text.push_str(
                &model
                    .token_to_piece(token, &mut decoder, false, None)
                    .map_err(runtime_error)?,
            );
            generated = generated.saturating_add(1);
            batch.clear();
            batch
                .add(token, position, &[0], true)
                .map_err(runtime_error)?;
            context.decode(&mut batch).map_err(runtime_error)?;
            position = position.saturating_add(1);
        }
        if text.trim().is_empty() {
            return Err(AppError::Provider(
                "Le modèle local a renvoyé une réponse vide.".into(),
            ));
        }
        Ok(RuntimeGeneration {
            text,
            prompt_tokens: u32::try_from(tokens.len()).unwrap_or(u32::MAX),
            generated_tokens: generated,
            load_time_ms,
            backend,
        })
    }

    fn load(
        &self,
        model_id: LocalModelId,
        path: &Path,
        expected_sha256: &str,
        requested_backend: LocalAiBackend,
    ) -> AppResult<(Arc<LlamaModel>, LocalAiBackend, u64)> {
        let llama_backend = shared_backend()?;
        let mut loaded = self
            .loaded
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(current) = loaded.as_ref() {
            if current.id == model_id && current.path == path {
                return Ok((Arc::clone(&current.model), current.backend, 0));
            }
        }
        loaded.take();
        if !path.is_file() {
            return Err(AppError::NotFound("modèle Mistral Local".into()));
        }
        let started = std::time::Instant::now();
        let digest = super::local_downloader::sha256_file(path)?;
        if !digest.eq_ignore_ascii_case(expected_sha256) {
            tracing::error!(model = ?model_id, "empreinte du modèle local modifiée depuis l'installation");
            return Err(AppError::Provider(
                "Le modèle local est endommagé. Supprimez-le puis réinstallez-le.".into(),
            ));
        }
        let actual_backend =
            if requested_backend == LocalAiBackend::Cpu || !llama_backend.supports_gpu_offload() {
                LocalAiBackend::Cpu
            } else {
                requested_backend
            };
        let params = if actual_backend == LocalAiBackend::Cpu {
            LlamaModelParams::default()
                .with_n_gpu_layers(0)
                .with_use_mmap(true)
        } else {
            LlamaModelParams::default()
                .with_n_gpu_layers(1_000)
                .with_use_mmap(true)
        };
        let model = Arc::new(
            LlamaModel::load_from_file(&llama_backend, path, &params).map_err(|error| {
                let detail = error.to_string();
                tracing::error!(model = ?model_id, backend = ?actual_backend, %detail, "modèle local non chargé");
                let lowered = detail.to_ascii_lowercase();
                if lowered.contains("out of memory")
                    || lowered.contains("oom")
                    || lowered.contains("failed to allocate")
                    || lowered.contains("cannot allocate")
                {
                    AppError::Provider(
                        "Mémoire insuffisante pour charger le modèle local. Fermez d'autres applications ou choisissez un profil plus léger.".into(),
                    )
                } else {
                    AppError::Provider("Le modèle local n'a pas pu être chargé.".into())
                }
            })?,
        );
        let load_time_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        tracing::info!(model = ?model_id, backend = ?actual_backend, load_time_ms, "modèle Mistral Local chargé");
        *loaded = Some(LoadedModel {
            id: model_id,
            path: path.to_path_buf(),
            model: Arc::clone(&model),
            backend: actual_backend,
        });
        Ok((model, actual_backend, load_time_ms))
    }
}

fn runtime_error(error: impl std::fmt::Display) -> AppError {
    let detail = error.to_string();
    tracing::error!(%detail, "inférence Mistral Local échouée");
    let lowered = detail.to_ascii_lowercase();
    if lowered.contains("out of memory")
        || lowered.contains("oom")
        || lowered.contains("failed to allocate")
        || lowered.contains("cannot allocate")
        || lowered.contains("std::bad_alloc")
    {
        return AppError::Provider(
            "Mémoire insuffisante pour l'IA locale. Fermez d'autres applications, choisissez un profil plus léger, ou raccourcissez le document.".into(),
        );
    }
    AppError::Provider(
        "L'inférence locale a échoué. Réessayez ou réévaluez la configuration.".into(),
    )
}
