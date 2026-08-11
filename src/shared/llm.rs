//! Configuration du fournisseur LLM, partagée entre les modules `settings` et `ia`.

use crate::shared::error::{AppError, AppResult};
use crate::shared::validation::is_local_or_private_ip;
use serde::{Deserialize, Serialize};

/// Fournisseur LLM sélectionné.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    /// Ollama local (défaut, privacy-first).
    Ollama,
    /// API Anthropic Claude.
    Claude,
    /// API `OpenAI`.
    OpenAI,
    /// API Google Gemini.
    Gemini,
    /// API Mistral.
    Mistral,
    /// API NVIDIA NIM / build.nvidia.com.
    Nvidia,
    /// Fournisseur personnalisé compatible `OpenAI` (nom libre).
    ///
    /// Note : ce variant se sérialise en `serde` (tag externe) sous la forme
    /// `{"custom":"nom"}`. Il n'est volontairement pas encore exposé dans le
    /// contrat frontend (`src/features/settings/model.ts`, enum `provider`) :
    /// l'implémentation de Custom est prévue en Tranche 2 (abstraction IA).
    Custom(String),
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ollama => formatter.write_str("Ollama"),
            Self::Claude => formatter.write_str("Claude"),
            Self::OpenAI => formatter.write_str("OpenAI"),
            Self::Gemini => formatter.write_str("Gemini"),
            Self::Mistral => formatter.write_str("Mistral"),
            Self::Nvidia => formatter.write_str("Nvidia"),
            Self::Custom(name) => write!(formatter, "Personnalisé ({name})"),
        }
    }
}

/// Mode d'analyse `IA`, adapté à la capacité du modèle.
///
/// Module l'ensemble du pipeline (budgets de tokens, fenêtre de contexte, nombre de
/// tentatives, validation anti-hallucination, parallélisme). `Auto` (défaut) déduit le
/// mode du fournisseur et du nom de modèle ; les trois autres forcent un comportement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisMode {
    /// Déduit automatiquement le mode du fournisseur/modèle (voir [`LlmConfig::resolved_mode`]).
    #[default]
    Auto,
    /// Petits modèles locaux (≈ 1B) : contexte réduit, une tâche par requête, tokens serrés,
    /// validation forte, aucune invention.
    Small,
    /// Modèles intermédiaires (≈ 3–8B) : réglages équilibrés (comportement historique).
    Standard,
    /// Gros modèles locaux ou fournisseurs cloud : contexte large, parallélisme, confiance accrue.
    Advanced,
}

impl std::fmt::Display for AnalysisMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Auto => "Automatique",
            Self::Small => "Petit modèle",
            Self::Standard => "Standard",
            Self::Advanced => "Avancé",
        })
    }
}

/// Configuration du fournisseur LLM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Fournisseur choisi.
    pub provider: ProviderKind,
    /// Clé API (facultative pour Ollama).
    pub api_key: Option<String>,
    /// Endpoint personnalisé (requis pour Custom).
    pub endpoint: Option<String>,
    /// Nom du modèle.
    pub model: String,
    /// Température d'échantillonnage (0.0–2.0).
    pub temperature: f32,
    /// Mode d'analyse. `#[serde(default)]` = rétro-compatible avec les configs persistées
    /// avant l'introduction du champ (elles retombent sur `Auto`).
    #[serde(default)]
    pub mode: AnalysisMode,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: ProviderKind::Ollama,
            api_key: None,
            endpoint: Some("http://localhost:11434".into()),
            model: "llama3.2:3b".into(),
            temperature: 0.7,
            mode: AnalysisMode::Auto,
        }
    }
}

impl LlmConfig {
    /// Résout le mode effectif : renvoie le mode forcé, ou en déduit un depuis le
    /// fournisseur et le nom de modèle quand il vaut [`AnalysisMode::Auto`].
    ///
    /// Heuristique (nom de modèle en minuscules) : les fournisseurs cloud et les gros
    /// modèles (`13b`+, `large`, `gpt-4`, `sonnet`, `opus`, `mixtral`…) → `Advanced` ; les
    /// modèles ≈ 1–2B (`1b`, `1.5b`, `2b`, `0.5b`, `mini`, `tiny`) → `Small` ; tout le reste
    /// (dont les Ollama non reconnus, ≈ 3–8B) → `Standard`, choix médian sûr.
    #[must_use]
    pub fn resolved_mode(&self) -> AnalysisMode {
        if self.mode != AnalysisMode::Auto {
            return self.mode;
        }
        // Les fournisseurs cloud servent des modèles puissants : mode avancé par défaut.
        if !matches!(self.provider, ProviderKind::Ollama) {
            return AnalysisMode::Advanced;
        }
        let m = self.model.to_lowercase();
        let has = |needles: &[&str]| needles.iter().any(|n| m.contains(n));
        if has(&[
            "70b",
            "72b",
            "34b",
            "32b",
            "13b",
            "14b",
            "large",
            "gpt-4",
            "sonnet",
            "opus",
            "mixtral",
            "command-r",
        ]) {
            AnalysisMode::Advanced
        } else if has(&[
            ":0.5b", ":1b", "-1b", ":1.5b", "-1.5b", ":2b", "-2b", "1.5b", "0.5b", "mini", "tiny",
            "small",
        ]) {
            AnalysisMode::Small
        } else {
            AnalysisMode::Standard
        }
    }

    /// Indique si la configuration est suffisamment renseignée pour tenter un appel.
    ///
    /// Ollama/Custom exigent un `endpoint` non vide ; les autres fournisseurs
    /// exigent une clé `API` non vide. Un modèle vide invalide toute configuration.
    /// Sert de garde-fou : le scoring du flux se désactive proprement si faux.
    #[must_use]
    pub fn est_configure(&self) -> bool {
        if self.model.trim().is_empty() {
            return false;
        }
        match &self.provider {
            ProviderKind::Ollama | ProviderKind::Custom(_) => self
                .endpoint
                .as_deref()
                .is_some_and(|e| !e.trim().is_empty()),
            _ => self
                .api_key
                .as_deref()
                .is_some_and(|k| !k.trim().is_empty()),
        }
    }
}

/// Valide l'endpoint réseau d'un fournisseur avant tout appel sortant.
///
/// Tous les fournisseurs distants doivent utiliser HTTPS et ne peuvent pas cibler
/// le réseau local. Ollama conserve volontairement l'accès local nécessaire à son usage.
/// Adresse retenue lors de la validation d'un endpoint, à épingler sur le client HTTP.
///
/// Ferme l'intervalle entre la vérification et l'usage : la connexion emprunte exactement
/// l'adresse qui a été contrôlée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointPin {
    /// Nom d'hôte tel qu'il figure dans l'URL.
    pub host: String,
    /// Adresse résolue et validée.
    pub address: std::net::SocketAddr,
}

///
/// # Errors
/// Retourne `Validation` si l'URL ou sa destination ne respecte pas la politique réseau.
#[allow(clippy::unnecessary_lazy_evaluations)]
pub async fn validate_llm_endpoint(config: &LlmConfig) -> AppResult<Option<EndpointPin>> {
    let Some(endpoint) = config.endpoint.as_deref() else {
        return Ok(None);
    };
    let url = reqwest::Url::parse(endpoint)
        .map_err(|_| AppError::Validation("Endpoint IA invalide".into()))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(AppError::Validation(
            "L'endpoint IA doit utiliser HTTP ou HTTPS".into(),
        ));
    }
    if !matches!(config.provider, ProviderKind::Ollama) {
        if url.scheme() != "https" {
            return Err(AppError::Validation(
                "Un endpoint IA distant doit utiliser HTTPS".into(),
            ));
        }
        let host = url.host_str().unwrap_or_default();
        if host.eq_ignore_ascii_case("localhost") || host.ends_with(".localhost") {
            return Err(AppError::Validation(
                "L'endpoint IA distant ne peut pas cibler le réseau local".into(),
            ));
        }
        let port = url.port_or_known_default().unwrap_or_else(|| 443);
        let addresses: Vec<std::net::SocketAddr> = tokio::net::lookup_host((host, port))
            .await
            .map_err(|_| AppError::Validation("Impossible de résoudre l'endpoint IA".into()))?
            .collect();
        if addresses
            .iter()
            .any(|address| is_local_or_private_ip(address.ip()))
        {
            return Err(AppError::Validation(
                "L'endpoint IA distant ne peut pas cibler une adresse privée".into(),
            ));
        }
        // L'adresse validée est renvoyée pour être épinglée sur le client HTTP. Sans cela, la
        // requête effective referait sa **propre** résolution : rien ne garantirait qu'elle
        // obtienne la même réponse, et un serveur DNS malveillant pourrait renvoyer une adresse
        // publique au contrôle puis une adresse privée à la connexion (« DNS rebinding »).
        if let Some(address) = addresses.first() {
            return Ok(Some(EndpointPin {
                host: host.to_owned(),
                address: *address,
            }));
        }
    }
    Ok(None)
}

#[cfg(test)]
#[path = "tests/llm/mod.rs"]
mod tests;
