//! Runtime llama.cpp embarqué, chargé paresseusement et sérialisé par processus.

use crate::core::errors::{AppError, AppResult};
use crate::features::ai::domain::{
    local_ai_memory_shortfall_mb, LocalAiBackend, LocalInferenceProgress, LocalModelId,
    ModelRegistry, LOCAL_AI_CONTEXT_SIZE,
};
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
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

const MAX_OUTPUT_TOKENS: usize = 4_096;
/// Plafond d'une sortie structurée.
///
/// Mesuré sur `tests/fixtures/profiles/` avec le tokeniseur du modèle : le profil valide le
/// plus riche pèse 1 928 jetons, et le cas volontairement trop long pour une page A4 en
/// pèse 2 634. Ce plafond les couvre tous avec 17 % de marge, sans laisser au modèle les
/// 4 096 jetons du texte libre — soit vingt-six minutes de divergence possible sur un
/// portable à 2,6 jetons/s.
const MAX_JSON_OUTPUT_TOKENS: usize = 3_072;
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

/// Délai d'inactivité au-delà duquel les poids sont rendus au système.
///
/// Un modèle chargé immobilise plusieurs gigaoctets. Les conserver après un import de CV
/// n'accélère qu'une éventuelle inférence suivante, au prix d'un risque d'arrêt par le
/// noyau pour tout le reste de la session.
pub const LOCAL_AI_IDLE_UNLOAD: Duration = Duration::from_secs(300);

#[derive(Default)]
pub struct MistralLocalRuntime {
    loaded: Mutex<Option<LoadedModel>>,
    inference: Mutex<()>,
    /// Instant de la dernière inférence. `None` quand rien n'est à libérer.
    last_used: Mutex<Option<Instant>>,
    /// Jeton de l'inférence en cours, lu par les boucles de décodage et de génération.
    current_inference: Mutex<Option<CancellationToken>>,
    /// Avancement de l'inférence en cours : jetons produits et instant de départ.
    progression: Mutex<Option<(u32, Instant)>>,
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
        self.last_used
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        tracing::info!("modèle Mistral Local libéré");
    }

    /// Ouvre une inférence et rend le jeton que ses boucles devront consulter.
    fn debuter_inference(&self) -> CancellationToken {
        let token = CancellationToken::new();
        *self
            .current_inference
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(token.clone());
        token
    }

    /// Referme l'inférence : plus rien n'est annulable jusqu'à la suivante.
    fn terminer_inference(&self) {
        self.current_inference
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        self.progression
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }

    /// Publie l'avancement de la génération, relu par l'écran d'import.
    fn noter_progression(&self, generes: u32, depuis: Instant) {
        *self
            .progression
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some((generes, depuis));
    }

    /// Avancement de l'inférence en cours, `None` si aucune n'est active.
    ///
    /// Douze minutes de silence ne se distinguent pas d'un blocage : cette mesure est la
    /// seule chose qui rende une attente aussi longue lisible.
    #[must_use]
    pub fn progression(&self) -> Option<LocalInferenceProgress> {
        let (generes, depuis) = (*self
            .progression
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner))?;
        let elapsed = depuis.elapsed();
        let elapsed_ms = u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX);
        let secondes = elapsed.as_secs_f32();
        Some(LocalInferenceProgress {
            generated_tokens: generes,
            elapsed_ms,
            tokens_per_second: if secondes > 0.0 {
                generes as f32 / secondes
            } else {
                0.0
            },
        })
    }

    /// Interrompt l'inférence en cours, s'il y en a une.
    ///
    /// Indispensable : la génération tourne dans un `spawn_blocking`, que Tokio ne sait pas
    /// interrompre. Sans ce drapeau, annuler rendait la main à l'interface pendant que les
    /// cœurs continuaient de calculer, parfois vingt minutes durant.
    pub fn cancel_inference(&self) {
        if let Some(token) = self
            .current_inference
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            token.cancel();
            tracing::info!("inférence locale interrompue à la demande");
        }
    }

    /// Enregistre la fin d'une inférence comme point de départ de l'inactivité.
    fn marquer_utilisation_a(&self, instant: Instant) {
        *self
            .last_used
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(instant);
    }

    /// Le modèle est-il inactif depuis au moins `delai` ?
    fn doit_liberer(&self, now: Instant, delai: Duration) -> bool {
        self.last_used
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some_and(|dernier| now.saturating_duration_since(dernier) >= delai)
    }

    /// Libère les poids si l'inactivité dépasse `delai`. Renvoie `true` si une libération
    /// a eu lieu.
    fn unload_if_idle_at(&self, now: Instant, delai: Duration) -> bool {
        if !self.doit_liberer(now, delai) {
            return false;
        }
        tracing::info!(
            inactivite_s = delai.as_secs(),
            "libération du modèle Mistral Local après inactivité"
        );
        self.unload();
        true
    }

    /// Point d'entrée de la surveillance périodique d'inactivité.
    pub fn unload_if_idle(&self, delai: Duration) -> bool {
        self.unload_if_idle_at(Instant::now(), delai)
    }

    /// Refuse l'inférence quand la RAM disponible ne laisse pas sa marge au système.
    ///
    /// Seule défense possible contre l'OOM killer : une fois la mémoire réservée, le
    /// `SIGKILL` du noyau n'est ni interceptable ni journalisable.
    fn ensure_memory(&self, model_id: LocalModelId, available_ram_mb: u64) -> AppResult<()> {
        let model = ModelRegistry::get(model_id)
            .ok_or_else(|| AppError::NotFound("modèle Mistral Local".into()))?;
        // Le chemin d'un identifiant est déterministe : comparer les identifiants suffit à
        // savoir si les poids occupent déjà la RAM mesurée.
        let already_loaded = self
            .loaded
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|current| current.id == model_id);
        let Some(manquant_mb) =
            local_ai_memory_shortfall_mb(&model, available_ram_mb, already_loaded)
        else {
            return Ok(());
        };
        tracing::warn!(
            model = ?model_id,
            available_ram_mb,
            manquant_mb,
            already_loaded,
            "inférence locale refusée faute de mémoire"
        );
        Err(AppError::Provider(format!(
            "Mémoire insuffisante pour l'IA locale : il manque environ {manquant_mb} Mo. \
             Fermez d'autres applications, ou choisissez un profil plus léger."
        )))
    }

    /// Exécute une inférence en la bornant par le garde-fou mémoire, et journalise
    /// l'empreinte de part et d'autre.
    ///
    /// Ces deux lignes de journal sont la seule trace exploitable après un arrêt brutal :
    /// un `SIGKILL` ne laisse rien écrire, seul l'encadrement permet de constater qu'une
    /// inférence était en cours au moment de la mort du processus.
    pub fn generate(&self, request: RuntimeRequest<'_>) -> AppResult<RuntimeGeneration> {
        let _inference = self
            .inference
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let model_id = request.model_id;
        let available_ram_mb = super::local_hardware::available_ram_mb();
        self.ensure_memory(model_id, available_ram_mb)?;
        let rss_avant_mb = super::local_hardware::process_rss_mb();
        tracing::info!(
            model = ?model_id,
            available_ram_mb,
            rss_avant_mb,
            "inférence locale démarrée"
        );
        let annulation = self.debuter_inference();
        let resultat = self.generate_inner(request, &annulation);
        self.terminer_inference();
        self.marquer_utilisation_a(Instant::now());
        tracing::info!(
            model = ?model_id,
            rss_apres_mb = super::local_hardware::process_rss_mb(),
            available_ram_mb = super::local_hardware::available_ram_mb(),
            succes = resultat.is_ok(),
            "inférence locale terminée"
        );
        resultat
    }

    fn generate_inner(
        &self,
        request: RuntimeRequest<'_>,
        annulation: &CancellationToken,
    ) -> AppResult<RuntimeGeneration> {
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
        let output_limit = plafond_de_sortie(request.max_output_tokens, request.json, remaining);
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
        let taille_lot = LOCAL_AI_BATCH_SIZE as usize;
        let mut batch = LlamaBatch::new(taille_lot, 1);
        let last = tokens.len().saturating_sub(1);
        // Les logits ne sont demandés que sur l'ultime jeton : c'est de lui seul que part
        // l'échantillonnage, et les réclamer partout gonflerait le tampon de sortie.
        for plage in lots_de_decodage(tokens.len(), taille_lot) {
            if annulation.is_cancelled() {
                return Err(AppError::Cancelled);
            }
            batch.clear();
            for index in plage {
                batch
                    .add(
                        tokens[index],
                        i32::try_from(index).map_err(runtime_error)?,
                        &[0],
                        index == last,
                    )
                    .map_err(runtime_error)?;
            }
            context.decode(&mut batch).map_err(runtime_error)?;
        }

        let debut_generation = Instant::now();
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
            if annulation.is_cancelled() {
                return Err(AppError::Cancelled);
            }
            // `sample` accepte déjà le jeton retenu (`llama_sampler_sample` appelle
            // `llama_sampler_accept`). L'accepter une seconde fois faisait avancer la
            // grammaire de deux pas par jeton : ses piles d'analyse finissaient vides, et
            // l'appel suivant abandonnait le processus sur `GGML_ASSERT(!stacks.empty())`.
            let token = sampler.sample(&context, batch.n_tokens() - 1);
            if model.is_eog_token(token) {
                break;
            }
            text.push_str(
                &model
                    .token_to_piece(token, &mut decoder, false, None)
                    .map_err(runtime_error)?,
            );
            generated = generated.saturating_add(1);
            self.noter_progression(generated, debut_generation);
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

/// Nombre maximal de jetons à produire, plafonds et contexte restant compris.
fn plafond_de_sortie(demande: Option<usize>, json: bool, restant: usize) -> usize {
    let plafond = if json {
        MAX_JSON_OUTPUT_TOKENS
    } else {
        MAX_OUTPUT_TOKENS
    };
    demande.unwrap_or(plafond).min(plafond).min(restant)
}

/// Découpe une invite de `total` jetons en plages décodables d'au plus `taille_lot`.
///
/// `llama_context::decode` impose `n_tokens_all <= n_batch` et **abandonne le processus**
/// par `GGML_ASSERT` au-delà : un `abort()` que Rust ne peut ni intercepter, ni convertir
/// en erreur, ni même journaliser. Décoder l'invite d'un seul bloc faisait donc planter
/// l'application pour tout texte dépassant 512 jetons — soit n'importe quel CV.
fn lots_de_decodage(total: usize, taille_lot: usize) -> Vec<std::ops::Range<usize>> {
    let taille_lot = taille_lot.max(1);
    (0..total)
        .step_by(taille_lot)
        .map(|debut| debut..(debut + taille_lot).min(total))
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::ai::domain::{LocalModelId, LOCAL_AI_SYSTEM_MARGIN_MB};
    use std::time::Duration;

    #[test]
    fn l_inference_est_refusee_quand_la_ram_disponible_ne_couvre_pas_le_modele() {
        let runtime = MistralLocalRuntime::new();
        let error = runtime
            .ensure_memory(LocalModelId::Ministral3Light, 512)
            .expect_err("le garde-fou doit refuser 512 Mo disponibles");
        let AppError::Provider(message) = error else {
            panic!("le refus mémoire doit être une erreur de fournisseur");
        };
        assert!(
            message.contains("Mémoire insuffisante"),
            "message inattendu : {message}"
        );
    }

    #[test]
    fn l_inference_est_autorisee_quand_la_marge_systeme_est_disponible() {
        let runtime = MistralLocalRuntime::new();
        let model = ModelRegistry::get(LocalModelId::Ministral3Light).expect("profil léger");
        let suffisant = model.estimated_ram_mb + LOCAL_AI_SYSTEM_MARGIN_MB;
        assert!(runtime
            .ensure_memory(LocalModelId::Ministral3Light, suffisant)
            .is_ok());
    }

    /// llama.cpp impose `n_tokens_all <= n_batch` par appel à `decode` et **abandonne le
    /// processus** (`GGML_ASSERT`) sinon — un `abort()` que Rust ne peut ni intercepter ni
    /// journaliser. L'invite doit donc être décodée par lots, jamais d'un bloc.
    /// Une sortie structurée n'a pas besoin du plafond du texte libre. Mesuré sur
    /// `tests/fixtures/profiles/` : le profil le plus volumineux — un cas volontairement
    /// trop long pour une page A4 — pèse 2 634 jetons, le plus riche des profils valides
    /// 1 928. À 2,6 jetons/s sur un portable, 4 096 jetons autorisaient vingt-six minutes
    /// de divergence.
    /// `spawn_blocking` ne s'interrompt pas : abandonner le futur rend la main à
    /// l'interface, mais la génération continue de saturer les cœurs. Seul un drapeau lu
    /// dans la boucle arrête réellement le calcul.
    /// Sans progression affichée, rien ne distingue « lent » de « planté » : une analyse de
    /// CV dure une douzaine de minutes sur un portable à 2,6 jetons/s.
    #[test]
    fn aucune_progression_hors_inference() {
        assert!(MistralLocalRuntime::new().progression().is_none());
    }

    #[test]
    fn la_progression_compte_les_jetons_et_leur_debit() {
        let runtime = MistralLocalRuntime::new();
        runtime.debuter_inference();

        let debut = Instant::now();
        runtime.noter_progression(52, debut - Duration::from_secs(20));
        let progression = runtime.progression().expect("inférence en cours");

        assert_eq!(progression.generated_tokens, 52);
        assert!(progression.elapsed_ms >= 20_000);
        assert!(
            (progression.tokens_per_second - 2.6).abs() < 0.1,
            "débit inattendu : {}",
            progression.tokens_per_second
        );
    }

    #[test]
    fn la_progression_disparait_avec_la_fin_de_l_inference() {
        let runtime = MistralLocalRuntime::new();
        runtime.debuter_inference();
        runtime.noter_progression(10, Instant::now());
        assert!(runtime.progression().is_some());

        runtime.terminer_inference();
        assert!(runtime.progression().is_none());
    }

    #[test]
    fn annuler_marque_l_inference_en_cours() {
        let runtime = MistralLocalRuntime::new();
        let token = runtime.debuter_inference();
        assert!(!token.is_cancelled(), "une inférence naît active");

        runtime.cancel_inference();
        assert!(
            token.is_cancelled(),
            "l'annulation doit atteindre la boucle"
        );
    }

    #[test]
    fn annuler_sans_inference_en_cours_est_sans_effet() {
        let runtime = MistralLocalRuntime::new();
        runtime.cancel_inference();
        assert!(!runtime.debuter_inference().is_cancelled());
    }

    #[test]
    fn une_nouvelle_inference_ne_herite_pas_de_l_annulation_precedente() {
        let runtime = MistralLocalRuntime::new();
        let premiere = runtime.debuter_inference();
        runtime.cancel_inference();
        runtime.terminer_inference();

        let seconde = runtime.debuter_inference();
        assert!(premiere.is_cancelled());
        assert!(
            !seconde.is_cancelled(),
            "l'annulation ne doit pas déteindre"
        );
    }

    #[test]
    fn une_sortie_json_est_plafonnee_plus_bas_que_le_texte_libre() {
        let restant = 8_000;
        assert_eq!(
            plafond_de_sortie(None, true, restant),
            MAX_JSON_OUTPUT_TOKENS
        );
        assert_eq!(plafond_de_sortie(None, false, restant), MAX_OUTPUT_TOKENS);
        const { assert!(MAX_JSON_OUTPUT_TOKENS < MAX_OUTPUT_TOKENS) };
    }

    /// Le plafond JSON doit couvrir le plus gros profil mesuré, marge comprise : le tronquer
    /// produirait un JSON invalide, c'est-à-dire un échec là où l'on avait un résultat lent.
    #[test]
    fn le_plafond_json_couvre_le_plus_gros_profil_mesure() {
        // Contrainte de compilation : abaisser le plafond sous le plus gros profil mesuré
        // tronquerait un JSON réel, transformant un résultat lent en échec.
        const {
            assert!(
                MAX_JSON_OUTPUT_TOKENS >= 2_634 + 256,
                "plafond trop serré pour un profil réel"
            )
        };
    }

    #[test]
    fn le_contexte_restant_prime_sur_le_plafond() {
        assert_eq!(plafond_de_sortie(None, true, 100), 100);
        assert_eq!(plafond_de_sortie(Some(64), false, 100), 64);
        assert_eq!(plafond_de_sortie(Some(9_000), false, 100), 100);
    }

    #[test]
    fn une_demande_explicite_ne_depasse_jamais_le_plafond() {
        assert_eq!(
            plafond_de_sortie(Some(usize::MAX), true, usize::MAX),
            MAX_JSON_OUTPUT_TOKENS
        );
    }

    #[test]
    fn aucun_lot_de_decodage_ne_depasse_la_taille_de_lot() {
        for total in [1_usize, 511, 512, 513, 1_024, 3_000, 8_000] {
            for lot in [1_usize, 64, 512] {
                for plage in lots_de_decodage(total, lot) {
                    assert!(
                        plage.len() <= lot,
                        "lot de {} jetons pour un maximum de {lot} (total {total})",
                        plage.len()
                    );
                }
            }
        }
    }

    #[test]
    fn les_lots_couvrent_toute_l_invite_dans_l_ordre() {
        let lots = lots_de_decodage(3_000, 512);
        assert_eq!(lots.len(), 6);
        let mut attendu = 0;
        for plage in &lots {
            assert_eq!(plage.start, attendu, "lots disjoints ou désordonnés");
            attendu = plage.end;
        }
        assert_eq!(attendu, 3_000, "la fin de l'invite doit être décodée");
    }

    #[test]
    fn une_invite_plus_courte_qu_un_lot_tient_en_un_seul_appel() {
        assert_eq!(lots_de_decodage(30, 512), vec![0..30]);
        assert_eq!(lots_de_decodage(512, 512), vec![0..512]);
        assert_eq!(lots_de_decodage(513, 512), vec![0..512, 512..513]);
    }

    #[test]
    fn une_invite_vide_ne_produit_aucun_lot() {
        assert!(lots_de_decodage(0, 512).is_empty());
    }

    #[test]
    fn un_modele_jamais_utilise_ne_declenche_pas_de_liberation() {
        let runtime = MistralLocalRuntime::new();
        assert!(!runtime.doit_liberer(Instant::now(), Duration::from_secs(300)));
    }

    #[test]
    fn un_modele_inactif_au_dela_du_delai_doit_etre_libere() {
        let runtime = MistralLocalRuntime::new();
        let debut = Instant::now();
        runtime.marquer_utilisation_a(debut);
        assert!(runtime.doit_liberer(debut + Duration::from_secs(301), Duration::from_secs(300)));
    }

    #[test]
    fn un_modele_utilise_recemment_reste_charge() {
        let runtime = MistralLocalRuntime::new();
        let debut = Instant::now();
        runtime.marquer_utilisation_a(debut);
        assert!(!runtime.doit_liberer(debut + Duration::from_secs(299), Duration::from_secs(300)));
    }

    #[test]
    fn la_liberation_reinitialise_le_suivi_d_inactivite() {
        let runtime = MistralLocalRuntime::new();
        let debut = Instant::now();
        runtime.marquer_utilisation_a(debut);
        let tard = debut + Duration::from_secs(600);
        assert!(runtime.unload_if_idle_at(tard, Duration::from_secs(300)));
        // Une fois libéré, plus rien à libérer tant qu'aucune inférence n'a eu lieu.
        assert!(!runtime.unload_if_idle_at(tard, Duration::from_secs(300)));
    }

    #[test]
    fn le_refus_memoire_precise_le_nombre_de_megaoctets_manquants() {
        let runtime = MistralLocalRuntime::new();
        let model = ModelRegistry::get(LocalModelId::Ministral3Light).expect("profil léger");
        let disponible = model.estimated_ram_mb + LOCAL_AI_SYSTEM_MARGIN_MB - 300;
        let error = runtime
            .ensure_memory(LocalModelId::Ministral3Light, disponible)
            .expect_err("300 Mo manquants doivent être refusés");
        assert!(
            error.to_string().contains("300"),
            "le manque doit être chiffré : {error}"
        );
    }
}
