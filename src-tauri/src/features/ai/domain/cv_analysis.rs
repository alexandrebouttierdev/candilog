//! Méthode d'analyse de CV (Vision / Texte) et capacités du modèle actif.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{ManagedModelId, ManagedModelRegistry, ProviderKind};

/// Préférence utilisateur pour l'import de CV.
///
/// `Vision` est le défaut recommandé dès que le modèle le permet. `Text` force
/// l'extraction PDF classique, sans tentative Vision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum CvAnalysisMethod {
    #[default]
    Vision,
    Text,
}

/// Méthode effectivement utilisée pour produire le résultat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum CvAnalysisMethodUsed {
    Vision,
    Text,
}

/// Capacités du modèle réellement sélectionné (pas seulement du fournisseur).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ModelCapabilities {
    pub vision: bool,
}

/// Décision d'orchestration : chemin à tenter en premier, et si un repli Texte est prévu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CvAnalysisPlan {
    pub primary: CvAnalysisMethodUsed,
    pub allow_text_fallback: bool,
}

/// Résout le plan d'analyse à partir de la préférence et des capacités.
#[must_use]
pub fn resolve_cv_analysis_plan(
    preferred: CvAnalysisMethod,
    capabilities: ModelCapabilities,
) -> CvAnalysisPlan {
    match preferred {
        CvAnalysisMethod::Text => CvAnalysisPlan {
            primary: CvAnalysisMethodUsed::Text,
            allow_text_fallback: false,
        },
        CvAnalysisMethod::Vision if capabilities.vision => CvAnalysisPlan {
            primary: CvAnalysisMethodUsed::Vision,
            allow_text_fallback: true,
        },
        CvAnalysisMethod::Vision => CvAnalysisPlan {
            primary: CvAnalysisMethodUsed::Text,
            allow_text_fallback: false,
        },
    }
}

/// Détecte la capacité Vision à partir du fournisseur, du tag/modèle et, le cas échéant,
/// des capacités rapportées par Ollama (`/api/show`).
#[must_use]
pub fn detect_model_capabilities(
    provider: &ProviderKind,
    model: &str,
    ollama_capabilities: Option<&[String]>,
) -> ModelCapabilities {
    if let Some(caps) = ollama_capabilities {
        if caps.iter().any(|cap| cap.eq_ignore_ascii_case("vision")) {
            return ModelCapabilities { vision: true };
        }
        // Capacités non vides sans « vision » : le runtime confirme l'absence.
        if !caps.is_empty()
            && matches!(provider, ProviderKind::CandilogLocal | ProviderKind::Ollama)
        {
            return ModelCapabilities { vision: false };
        }
    }

    ModelCapabilities {
        vision: model_name_suggests_vision(provider, model),
    }
}

/// Heuristique fiable pour le catalogue Candilog et les familles multimodales connues.
#[must_use]
pub fn model_name_suggests_vision(provider: &ProviderKind, model: &str) -> bool {
    let normalized = model.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return false;
    }

    if let Some(definition) = ManagedModelRegistry::by_tag(&normalized) {
        return managed_model_supports_vision(definition.id);
    }
    // Tag partiel (quantization suffix, namespace).
    if let Some(id) = managed_id_from_partial_tag(&normalized) {
        return managed_model_supports_vision(id);
    }

    if looks_like_vision_family(&normalized) {
        return true;
    }

    // Fournisseurs cloud : familles multimodales courantes, sans tout autoriser.
    match provider {
        ProviderKind::CandilogLocal | ProviderKind::Ollama => false,
        ProviderKind::Claude => {
            normalized.contains("claude-3")
                || normalized.contains("claude-4")
                || normalized.contains("claude-sonnet")
                || normalized.contains("claude-opus")
                || normalized.contains("claude-haiku")
        }
        ProviderKind::Gemini => normalized.contains("gemini") || normalized.contains("gemma"),
        ProviderKind::OpenAI | ProviderKind::Custom(_) => {
            normalized.contains("gpt-4o")
                || normalized.contains("gpt-4.1")
                || normalized.contains("gpt-5")
                || normalized.contains("o1")
                || normalized.contains("o3")
                || normalized.contains("o4")
                || normalized.contains("vision")
        }
        ProviderKind::Mistral => {
            normalized.contains("pixtral")
                || normalized.contains("ministral")
                || normalized.contains("mistral-small")
                || normalized.contains("mistral-medium")
                || normalized.contains("mistral-large")
        }
        ProviderKind::DeepSeek => false,
    }
}

#[must_use]
pub fn managed_model_supports_vision(id: ManagedModelId) -> bool {
    matches!(
        id,
        ManagedModelId::Ministral3Light
            | ManagedModelId::Ministral3Balanced
            | ManagedModelId::Ministral3Powerful
    )
}

fn managed_id_from_partial_tag(tag: &str) -> Option<ManagedModelId> {
    for definition in ManagedModelRegistry::all() {
        let catalog = definition.ollama_tag.to_ascii_lowercase();
        if tag == catalog || tag.starts_with(&format!("{catalog}-")) || tag.ends_with(&catalog) {
            return Some(definition.id);
        }
        // `ministral-3:3b-q4_K_M` etc.
        if catalog.contains(':') {
            let (family, size) = catalog.split_once(':')?;
            if tag.starts_with(family) && tag.contains(size) {
                return Some(definition.id);
            }
        }
    }
    None
}

fn looks_like_vision_family(model: &str) -> bool {
    const MARKERS: &[&str] = &[
        "ministral-3",
        "ministral3",
        "pixtral",
        "llava",
        "bakllava",
        "moondream",
        "qwen2-vl",
        "qwen2.5-vl",
        "qwen3-vl",
        "llama3.2-vision",
        "llama-3.2-vision",
        "gemma3",
        "mistral-small3",
        "mistral-large-3",
        "devstral",
        "vision",
    ];
    MARKERS.iter().any(|marker| model.contains(marker))
}

/// Nombre maximal de pages envoyées au modèle Vision (CV raisonnables).
pub const MAX_VISION_PAGES: usize = 4;

/// DPI du rendu `pdftoppm` : lisible pour les petits caractères, sans images gigantesques.
pub const VISION_RENDER_DPI: u32 = 160;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vision_par_defaut_si_modele_compatible() {
        let plan =
            resolve_cv_analysis_plan(CvAnalysisMethod::Vision, ModelCapabilities { vision: true });
        assert_eq!(plan.primary, CvAnalysisMethodUsed::Vision);
        assert!(plan.allow_text_fallback);
    }

    #[test]
    fn modele_sans_vision_bascule_en_texte() {
        let plan = resolve_cv_analysis_plan(
            CvAnalysisMethod::Vision,
            ModelCapabilities { vision: false },
        );
        assert_eq!(plan.primary, CvAnalysisMethodUsed::Text);
        assert!(!plan.allow_text_fallback);
    }

    #[test]
    fn texte_force_ne_tente_jamais_la_vision() {
        let plan =
            resolve_cv_analysis_plan(CvAnalysisMethod::Text, ModelCapabilities { vision: true });
        assert_eq!(plan.primary, CvAnalysisMethodUsed::Text);
        assert!(!plan.allow_text_fallback);
    }

    #[test]
    fn ministral_catalogue_est_vision() {
        assert!(managed_model_supports_vision(
            ManagedModelId::Ministral3Light
        ));
        assert!(
            detect_model_capabilities(&ProviderKind::CandilogLocal, "ministral-3:3b", None).vision
        );
    }

    #[test]
    fn lfm_catalogue_n_est_pas_vision() {
        assert!(!managed_model_supports_vision(
            ManagedModelId::Lfm25UltraLight
        ));
        assert!(
            !detect_model_capabilities(&ProviderKind::CandilogLocal, "lfm2.5:1.2b", None).vision
        );
    }

    #[test]
    fn ollama_show_vision_prime_sur_le_nom() {
        let caps = detect_model_capabilities(
            &ProviderKind::Ollama,
            "custom-local:latest",
            Some(&["completion".into(), "vision".into()]),
        );
        assert!(caps.vision);
    }

    #[test]
    fn ollama_show_sans_vision_refuse_meme_un_nom_suggestif() {
        // Si le runtime confirme l'absence, on ne force pas Vision sur un tag custom.
        let caps = detect_model_capabilities(
            &ProviderKind::Ollama,
            "weird-vision-name:latest",
            Some(&["completion".into()]),
        );
        assert!(!caps.vision);
    }

    #[test]
    fn openai_gpt4o_est_vision() {
        assert!(detect_model_capabilities(&ProviderKind::OpenAI, "gpt-4o", None).vision);
    }
}
