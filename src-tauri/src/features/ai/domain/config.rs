//! Configuration compatible avec le JSON historique de `parametres`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Mode d'analyse, rétro-compatible avec les bases antérieures à son introduction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "settings.ts")]
pub enum AnalysisMode {
    #[default]
    Auto,
    Small,
    Standard,
    Advanced,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "settings.ts")]
pub enum ProviderKind {
    /// IA locale Candilog via runtime Ollama privé géré par l'application.
    CandilogLocal,
    Ollama,
    Claude,
    #[serde(rename = "openai", alias = "open_ai", alias = "OpenAI")]
    OpenAI,
    Gemini,
    Mistral,
    #[serde(rename = "deepseek", alias = "nvidia")]
    DeepSeek,
    Custom(String),
}

/// Champs en snake_case : c'est le JSON persisté par l'application Iced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: ProviderKind,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub model: String,
    pub temperature: f32,
    #[serde(default)]
    pub mode: AnalysisMode,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: ProviderKind::CandilogLocal,
            api_key: None,
            endpoint: None,
            model: String::new(),
            temperature: 0.7,
            mode: AnalysisMode::Auto,
        }
    }
}

impl LlmConfig {
    /// Réécrit un ancien enregistrement NVIDIA (alias `nvidia` → DeepSeek).
    pub fn normaliser_legacy(&mut self) {
        if !matches!(self.provider, ProviderKind::DeepSeek) {
            return;
        }
        if self.endpoint.as_deref() == Some("https://integrate.api.nvidia.com") {
            self.endpoint = Some("https://api.deepseek.com".into());
        }
        if self.model == "meta/llama-3.1-70b-instruct" {
            self.model = "deepseek-v4-flash".into();
        }
    }

    #[must_use]
    pub fn endpoint_effectif(&self) -> &str {
        self.endpoint.as_deref().unwrap_or(match self.provider {
            ProviderKind::CandilogLocal => "",
            ProviderKind::Ollama => "http://localhost:11434",
            ProviderKind::Claude => "https://api.anthropic.com",
            ProviderKind::Gemini => "https://generativelanguage.googleapis.com",
            ProviderKind::Mistral => "https://api.mistral.ai",
            ProviderKind::DeepSeek => "https://api.deepseek.com",
            ProviderKind::OpenAI | ProviderKind::Custom(_) => "https://api.openai.com",
        })
    }

    #[must_use]
    pub fn est_configure(&self) -> bool {
        match self.provider {
            ProviderKind::CandilogLocal => true,
            ProviderKind::Ollama => {
                !self.model.trim().is_empty() && !self.endpoint_effectif().trim().is_empty()
            }
            _ => {
                !self.model.trim().is_empty()
                    && self
                        .api_key
                        .as_deref()
                        .is_some_and(|key| !key.trim().is_empty())
            }
        }
    }
}

#[derive(Deserialize)]
pub struct SettingsStockes {
    /// Absent sur une base neuve (`data = '{}'`), présent dès que les réglages ont été sauvés.
    #[serde(default)]
    pub llm: LlmConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(provider: ProviderKind, model: &str) -> LlmConfig {
        LlmConfig {
            provider,
            api_key: None,
            endpoint: None,
            model: model.into(),
            temperature: 0.7,
            mode: AnalysisMode::Auto,
        }
    }

    #[test]
    fn candilog_local_reste_configure_sans_nom_de_modele() {
        assert!(config(ProviderKind::CandilogLocal, "").est_configure());
    }

    #[test]
    fn ollama_exige_toujours_un_modele() {
        assert!(!config(ProviderKind::Ollama, "").est_configure());
        assert!(config(ProviderKind::Ollama, "llama3.2:3b").est_configure());
    }

    #[test]
    fn un_fournisseur_cloud_exige_toujours_un_modele_et_une_cle() {
        assert!(!config(ProviderKind::Claude, "").est_configure());
        let mut avec_modele = config(ProviderKind::Claude, "claude-sonnet-4-0");
        assert!(
            !avec_modele.est_configure(),
            "sans clé, rien n'est configuré"
        );
        avec_modele.api_key = Some("sk-test".into());
        assert!(avec_modele.est_configure());
    }

    #[test]
    fn nvidia_est_relu_comme_deepseek() {
        let config: LlmConfig = serde_json::from_str(
            r#"{
                "provider": "nvidia",
                "api_key": null,
                "endpoint": "https://integrate.api.nvidia.com",
                "model": "meta/llama-3.1-70b-instruct",
                "temperature": 0.7
            }"#,
        )
        .unwrap();
        assert_eq!(config.provider, ProviderKind::DeepSeek);

        let mut config = config;
        config.normaliser_legacy();
        assert_eq!(config.endpoint.as_deref(), Some("https://api.deepseek.com"));
        assert_eq!(config.model, "deepseek-v4-flash");
    }
}
