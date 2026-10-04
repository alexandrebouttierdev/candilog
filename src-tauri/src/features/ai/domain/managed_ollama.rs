//! Contrats du runtime Ollama géré par Candilog et catalogue des modèles locaux recommandés.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const MANAGED_OLLAMA_PREFERRED_PORT: u16 = 11_435;
pub const MANAGED_OLLAMA_RUNTIME_VERSION: &str = "0.13.4";

/// Version du benchmark utilisateur `CV_BENCHMARK.pdf` — à incrémenter si le PDF, la ground
/// truth ou l'algorithme de scoring change de façon incompatible.
pub const USER_BENCHMARK_VERSION: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum ManagedModelId {
    Ministral3Light,
    Gemma4E2b,
    Ministral3Balanced,
    Gemma4E4b,
    Ministral3Powerful,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum ManagedModelCategory {
    UltraLight,
    Light,
    Balanced,
    Powerful,
    MaxQuality,
}

/// Éditeur du modèle Ollama — pilote le logo affiché sur les cartes du catalogue local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum ManagedModelPublisher {
    Mistral,
    Google,
}

impl ManagedModelPublisher {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Google => "Google DeepMind",
            Self::Mistral => "Mistral",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum MachineFit {
    Recommended,
    Compatible,
    MayBeSlow,
    InsufficientMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum ManagedRuntimeState {
    #[default]
    NotInstalled,
    Downloading,
    Installing,
    Starting,
    Ready,
    Stopping,
    Stopped,
    Updating,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ManagedModelDefinition {
    pub id: ManagedModelId,
    pub category: ManagedModelCategory,
    pub publisher: ManagedModelPublisher,
    pub publisher_label: String,
    pub display_name: String,
    pub description: String,
    /// Tag Ollama utilisé pour pull et inférence.
    pub ollama_tag: String,
    #[ts(type = "number")]
    pub approximate_download_bytes: u64,
    #[ts(type = "number")]
    pub recommended_ram_gb: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ManagedModelStatus {
    pub definition: ManagedModelDefinition,
    pub installed: bool,
    pub active: bool,
    pub machine_fit: MachineFit,
    pub recommended: bool,
    pub last_benchmark: Option<UserBenchmarkSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct UserBenchmarkSummary {
    pub score: u32,
    pub total_ms: u32,
    pub benchmark_version: u32,
    pub measured_at: String,
}

/// Réponse du modèle local actif à la phrase de test de l'installation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct LocalModelProbe {
    /// Tag Ollama du modèle qui a répondu.
    pub model: String,
    /// Aller-retour mesuré, chargement du modèle en mémoire compris.
    pub latency_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ManagedOllamaStatus {
    pub runtime_state: ManagedRuntimeState,
    pub runtime_version: Option<String>,
    pub port: Option<u16>,
    #[ts(type = "number")]
    pub models_disk_bytes: u64,
    pub active_model: Option<ManagedModelDefinition>,
    pub models: Vec<ManagedModelStatus>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ManagedOllamaDownloadProgress {
    pub kind: ManagedDownloadKind,
    pub model_id: Option<ManagedModelId>,
    pub state: ManagedRuntimeState,
    #[ts(type = "number")]
    pub downloaded_bytes: u64,
    #[ts(type = "number")]
    pub total_bytes: u64,
    pub progress: u8,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum ManagedDownloadKind {
    Runtime,
    Model,
}

/// Lit un identifiant de modèle en ramenant toute valeur hors catalogue à `None`.
fn modele_actif_tolerant<'de, D>(deserializer: D) -> Result<Option<ManagedModelId>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let brut = Option::<serde_json::Value>::deserialize(deserializer)?;
    let Some(brut) = brut.filter(|valeur| !valeur.is_null()) else {
        return Ok(None);
    };
    match serde_json::from_value::<ManagedModelId>(brut.clone()) {
        Ok(id) => Ok(Some(id)),
        Err(_) => {
            tracing::info!(
                valeur = %brut,
                "modèle local actif absent du catalogue : sélection remise à zéro"
            );
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ManagedOllamaSettings {
    /// Modèle actif, oublié plutôt que fatal s'il a quitté le catalogue.
    ///
    /// `#[serde(default)]` ne couvre que le champ **absent** : une valeur devenue inconnue —
    /// un modèle retiré du catalogue, comme les LFM2.5 — ferait échouer la désérialisation de
    /// tout le bloc de réglages, que le dépôt archiverait alors en `parametres_corrompus`.
    /// L'utilisateur perdrait ses clés, son routage et ses préférences pour un seul champ.
    /// Ici l'inconnu retombe sur `None` : l'écran redemande simplement quel modèle activer.
    #[serde(default, deserialize_with = "modele_actif_tolerant")]
    pub active_model_id: Option<ManagedModelId>,
    #[serde(default)]
    pub installed_model_tags: Vec<String>,
    #[serde(default)]
    pub runtime_state: ManagedRuntimeState,
    #[serde(default)]
    pub runtime_port: Option<u16>,
    #[serde(default)]
    pub runtime_version: Option<String>,
    #[serde(default)]
    pub benchmark_history: Vec<StoredBenchmarkResult>,
    #[serde(default)]
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct StoredBenchmarkResult {
    pub provider: String,
    pub model: String,
    pub benchmark_version: u32,
    pub score: u32,
    pub total_ms: u32,
    pub measured_at: String,
}

/// Catégories de score utilisateur pour l'affichage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum BenchmarkQualityLabel {
    Weak,
    Average,
    Fair,
    Good,
    VeryGood,
    Excellent,
}

#[must_use]
pub fn benchmark_quality_label(score: u32) -> BenchmarkQualityLabel {
    match score {
        0..=49 => BenchmarkQualityLabel::Weak,
        50..=64 => BenchmarkQualityLabel::Average,
        65..=74 => BenchmarkQualityLabel::Fair,
        75..=84 => BenchmarkQualityLabel::Good,
        85..=94 => BenchmarkQualityLabel::VeryGood,
        _ => BenchmarkQualityLabel::Excellent,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct UserBenchmarkCategoryScore {
    pub label: String,
    pub score: u32,
    pub max_score: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct UserBenchmarkMetrics {
    pub total_ms: u32,
    pub pdf_extract_ms: u32,
    pub preprocess_ms: u32,
    pub llm_ms: u32,
    pub parse_ms: u32,
    pub llm_calls: u32,
    pub tokens_input: Option<u32>,
    pub tokens_output: Option<u32>,
    pub tokens_per_second: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct UserBenchmarkResult {
    pub score: u32,
    pub quality: BenchmarkQualityLabel,
    pub metrics: UserBenchmarkMetrics,
    pub categories: Vec<UserBenchmarkCategoryScore>,
    pub hallucination_count: u32,
    pub benchmark_version: u32,
    pub provider_label: String,
    pub model_label: String,
    pub remote_warning: bool,
    /// Pipeline réellement exécuté pour ce run.
    pub method_used: super::CvAnalysisMethodUsed,
    pub fallback_used: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct InstallManagedModelRequest {
    pub model_id: ManagedModelId,
}

pub struct ManagedModelRegistry;

impl ManagedModelRegistry {
    #[must_use]
    pub fn all() -> Vec<ManagedModelDefinition> {
        vec![
            ManagedModelDefinition {
                id: ManagedModelId::Ministral3Light,
                category: ManagedModelCategory::Light,
                publisher: ManagedModelPublisher::Mistral,
                publisher_label: ManagedModelPublisher::Mistral.label().into(),
                display_name: "Ministral 3 3B".into(),
                description:
                    "3 milliards de paramètres, fenêtre de 256 000 jetons, texte et image. Le plus \
                     rapide du catalogue et le plus sobre en mémoire : c'est le choix par défaut \
                     pour les lettres, les CV et les analyses. Mistral AI le publie sous \
                     Apache 2.0 et annonce le français parmi ses langues principales."
                        .into(),
                ollama_tag: "ministral-3:3b".into(),
                approximate_download_bytes: 3_000_000_000,
                recommended_ram_gb: 8,
            },
            ManagedModelDefinition {
                id: ManagedModelId::Gemma4E2b,
                category: ManagedModelCategory::Balanced,
                publisher: ManagedModelPublisher::Google,
                publisher_label: ManagedModelPublisher::Google.label().into(),
                display_name: "Gemma 4 E2B".into(),
                description:
                    "2,3 milliards de paramètres effectifs (5,1 avec les embeddings), fenêtre de \
                     128 000 jetons. Texte, image et son, avec un encodeur visuel dédié d'environ \
                     150 millions de paramètres : c'est celui du catalogue dont l'outillage image \
                     est le plus explicite, pour l'import de CV en mode Vision. Publié par \
                     Google DeepMind."
                        .into(),
                ollama_tag: "gemma4:e2b".into(),
                approximate_download_bytes: 7_500_000_000,
                recommended_ram_gb: 16,
            },
            ManagedModelDefinition {
                id: ManagedModelId::Ministral3Balanced,
                category: ManagedModelCategory::Balanced,
                publisher: ManagedModelPublisher::Mistral,
                publisher_label: ManagedModelPublisher::Mistral.label().into(),
                display_name: "Ministral 3 8B".into(),
                description:
                    "8 milliards de paramètres, fenêtre de 256 000 jetons, texte et image. Deux \
                     fois plus de paramètres que le 3B, pour deux fois plus de mémoire : à \
                     préférer si les offres et les CV que vous traitez sont longs. Apache 2.0, \
                     français parmi les langues principales."
                        .into(),
                ollama_tag: "ministral-3:8b".into(),
                approximate_download_bytes: 6_000_000_000,
                recommended_ram_gb: 16,
            },
            ManagedModelDefinition {
                id: ManagedModelId::Gemma4E4b,
                category: ManagedModelCategory::MaxQuality,
                publisher: ManagedModelPublisher::Google,
                publisher_label: ManagedModelPublisher::Google.label().into(),
                display_name: "Gemma 4 E4B".into(),
                description:
                    "4,5 milliards de paramètres effectifs (8 avec les embeddings), fenêtre de \
                     128 000 jetons, texte et image. Le plus lourd à télécharger du catalogue, \
                     pour une empreinte mémoire comparable au Ministral 14B. Publié par \
                     Google DeepMind."
                        .into(),
                ollama_tag: "gemma4:e4b".into(),
                approximate_download_bytes: 9_500_000_000,
                recommended_ram_gb: 24,
            },
            ManagedModelDefinition {
                id: ManagedModelId::Ministral3Powerful,
                category: ManagedModelCategory::Powerful,
                publisher: ManagedModelPublisher::Mistral,
                publisher_label: ManagedModelPublisher::Mistral.label().into(),
                display_name: "Ministral 3 14B".into(),
                description:
                    "14 milliards de paramètres, fenêtre de 256 000 jetons, texte et image. Le \
                     plus grand du catalogue : la rédaction la plus détaillée, au prix de la \
                     vitesse et de 24 Gio de mémoire. Apache 2.0, français parmi les langues \
                     principales."
                        .into(),
                ollama_tag: "ministral-3:14b".into(),
                approximate_download_bytes: 9_100_000_000,
                recommended_ram_gb: 24,
            },
        ]
    }

    #[must_use]
    pub fn get(id: ManagedModelId) -> Option<ManagedModelDefinition> {
        Self::all().into_iter().find(|model| model.id == id)
    }

    #[must_use]
    pub fn by_tag(tag: &str) -> Option<ManagedModelDefinition> {
        let normalized = tag.trim();
        Self::all()
            .into_iter()
            .find(|model| model.ollama_tag == normalized)
    }
}

#[must_use]
pub fn evaluate_machine_fit(
    model: &ManagedModelDefinition,
    total_ram_gb: u32,
) -> (MachineFit, bool) {
    let fits = total_ram_gb >= model.recommended_ram_gb;
    let fit = if fits {
        MachineFit::Recommended
    } else if total_ram_gb >= model.recommended_ram_gb.saturating_sub(2).max(4) {
        MachineFit::Compatible
    } else if total_ram_gb >= model.recommended_ram_gb / 2 {
        MachineFit::MayBeSlow
    } else {
        MachineFit::InsufficientMemory
    };
    // `recommended` est recalculé au niveau catalogue : un seul modèle mis en avant.
    (fit, false)
}

/// Ordre de préférence pour le badge « Recommandé » : Ministral 3 3B d'abord quand la
/// machine le tient, sinon le plus grand modèle encore confortable.
#[must_use]
pub fn preferred_model_order() -> &'static [ManagedModelId] {
    &[
        ManagedModelId::Ministral3Light,
        ManagedModelId::Gemma4E2b,
        ManagedModelId::Ministral3Balanced,
        ManagedModelId::Gemma4E4b,
        ManagedModelId::Ministral3Powerful,
    ]
}

/// Choisit le modèle à badge « Recommandé » pour une quantité de RAM donnée.
#[must_use]
pub fn recommended_model_id(total_ram_gb: u32) -> Option<ManagedModelId> {
    preferred_model_order().iter().copied().find(|&id| {
        ManagedModelRegistry::get(id)
            .map(|model| {
                let (fit, _) = evaluate_machine_fit(&model, total_ram_gb);
                matches!(fit, MachineFit::Recommended | MachineFit::Compatible)
            })
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ministral_3b_est_recommande_des_8_go() {
        assert_eq!(
            recommended_model_id(8),
            Some(ManagedModelId::Ministral3Light)
        );
        assert_eq!(
            recommended_model_id(16),
            Some(ManagedModelId::Ministral3Light)
        );
    }

    #[test]
    fn une_machine_modeste_n_a_aucun_modele_recommande() {
        // Le catalogue n'a plus de modèle sous 8 Gio depuis le retrait des LFM2.5 : mieux
        // vaut ne rien recommander que pousser un modèle qui ne tiendra pas en mémoire.
        assert_eq!(recommended_model_id(2), None);
    }

    #[test]
    fn chaque_modele_du_catalogue_est_ordonne_et_atteignable() {
        let catalogue = ManagedModelRegistry::all();
        let ordre = preferred_model_order();

        assert_eq!(
            catalogue.len(),
            ordre.len(),
            "l'ordre de préférence doit couvrir exactement le catalogue"
        );
        for definition in &catalogue {
            assert!(
                ordre.contains(&definition.id),
                "{} absent de l'ordre de préférence",
                definition.display_name
            );
            assert!(
                ManagedModelRegistry::get(definition.id).is_some(),
                "{} introuvable par son identifiant",
                definition.display_name
            );
        }
    }

    #[test]
    fn les_tags_ollama_sont_uniques_et_renseignes() {
        let mut tags: Vec<String> = ManagedModelRegistry::all()
            .into_iter()
            .map(|definition| definition.ollama_tag)
            .collect();
        assert!(
            tags.iter().all(|tag| tag.contains(':')),
            "un tag Ollama porte toujours une taille : {tags:?}"
        );
        tags.sort();
        let avant = tags.len();
        tags.dedup();
        assert_eq!(avant, tags.len(), "deux modèles partagent le même tag");
    }

    #[test]
    fn un_modele_retire_du_catalogue_ne_corrompt_pas_les_reglages() {
        // Scénario d'une base existante : l'utilisateur avait activé un LFM2.5, que le
        // catalogue ne porte plus. Sans tolérance, tout le bloc de réglages devenait
        // illisible et partait en `parametres_corrompus` — clés et routage avec lui.
        let json =
            r#"{"active_model_id":"lfm25_ultra_light","installed_model_tags":["lfm2.5:1.2b"]}"#;

        let settings: ManagedOllamaSettings = serde_json::from_str(json)
            .expect("des réglages portant un modèle retiré restent lisibles");

        assert_eq!(settings.active_model_id, None);
        // Ce qui est sur le disque y reste : l'utilisateur le retire depuis Ollama.
        assert_eq!(
            settings.installed_model_tags,
            vec!["lfm2.5:1.2b".to_owned()]
        );
    }

    #[test]
    fn un_modele_du_catalogue_est_relu_normalement() {
        let json = r#"{"active_model_id":"gemma4_e4b"}"#;

        let settings: ManagedOllamaSettings = serde_json::from_str(json).unwrap();

        assert_eq!(settings.active_model_id, Some(ManagedModelId::Gemma4E4b));
    }
}
