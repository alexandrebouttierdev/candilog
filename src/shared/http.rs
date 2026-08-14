//! Construction centralisée des clients HTTP sortants.

use crate::shared::error::{AppError, AppResult};
use serde::de::DeserializeOwned;

/// Taille maximale d'une réponse JSON provenant d'un fournisseur IA.
pub const MAX_PROVIDER_RESPONSE_BYTES: usize = 5 * 1024 * 1024;

/// Durée maximale d'une génération LLM complète. Les modèles locaux peu puissants
/// peuvent légitimement dépasser le délai HTTP général de 45 secondes.
pub const PROVIDER_GENERATION_TIMEOUT: std::time::Duration = std::time::Duration::from_mins(30);

/// Construit un client avec délais bornés et redirections limitées.
#[must_use]
pub fn client() -> reqwest::Client {
    client_pinned(None)
}

/// Variante épinglée sur l'adresse validée par `validate_llm_endpoint`.
///
/// L'épinglage supprime la seconde résolution DNS qu'effectuerait la requête : la connexion
/// emprunte exactement l'adresse contrôlée, ce qui ferme la fenêtre de « DNS rebinding » entre
/// la vérification anti-SSRF et l'usage.
#[must_use]
pub fn client_pinned(pin: Option<&crate::shared::llm::EndpointPin>) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(8))
        .timeout(std::time::Duration::from_secs(45))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Candilog/0.1");
    if let Some(pin) = pin {
        builder = builder.resolve(&pin.host, pin.address);
    }
    builder.build().unwrap_or_else(|_| reqwest::Client::new())
}

/// Client de téléchargement de gros fichiers.
///
/// Conserve les protections de [`client`] — délai de connexion — mais abandonne le délai
/// **global**, incompatible avec un paquet de plusieurs dizaines de mégaoctets sur une liaison
/// lente.
///
/// Les redirections sont **suivies** : les assets d'une release GitHub sont servis après une
/// redirection vers le CDN propriétaire (`objects.githubusercontent.com`). L'URL provient de
/// la réponse de l'API GitHub, elle-même reçue en HTTPS.
#[must_use]
pub fn download_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(8))
        .read_timeout(std::time::Duration::from_secs(60))
        .user_agent("Candilog/0.1")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

/// Désérialise une réponse JSON sans permettre à un serveur distant de remplir la mémoire.
///
/// # Errors
/// Retourne une erreur si la réponse dépasse `limit`, si sa lecture échoue ou si le JSON est invalide.
pub async fn json_limited<T: DeserializeOwned>(
    mut response: reqwest::Response,
    limit: usize,
) -> AppResult<T> {
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err(AppError::Provider(
            "réponse distante trop volumineuse".into(),
        ));
    }

    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(AppError::from)? {
        if body.len().saturating_add(chunk.len()) > limit {
            return Err(AppError::Provider(
                "réponse distante trop volumineuse".into(),
            ));
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body).map_err(AppError::from)
}

/// Lit le corps d'une réponse **en streaming**, ligne par ligne, en appliquant `on_line` à chaque
/// ligne complète (sans le `\n`/`\r`). Utilisé pour décoder les flux NDJSON (Ollama) et SSE (cloud).
///
/// Le découpage se fait sur l'octet `\n` (jamais un octet de continuation UTF-8) : chaque ligne est
/// donc décodée intacte. La taille totale lue est bornée pour éviter la saturation mémoire.
///
/// # Errors
/// `AppError::Provider` si le flux dépasse la limite ; `AppError::Http` si la lecture réseau échoue.
pub async fn read_lines_stream(
    mut response: reqwest::Response,
    mut on_line: impl FnMut(&str) + Send,
) -> AppResult<()> {
    let mut buf: Vec<u8> = Vec::new();
    let mut total = 0usize;
    while let Some(chunk) = response.chunk().await.map_err(AppError::from)? {
        total = total.saturating_add(chunk.len());
        if total > MAX_PROVIDER_RESPONSE_BYTES {
            return Err(AppError::Provider(
                "réponse distante trop volumineuse".into(),
            ));
        }
        buf.extend_from_slice(&chunk);
        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buf.drain(..=pos).collect();
            on_line(String::from_utf8_lossy(&line).trim_end_matches(['\n', '\r']));
        }
    }
    if !buf.is_empty() {
        on_line(String::from_utf8_lossy(&buf).trim_end_matches(['\n', '\r']));
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/http/mod.rs"]
mod tests;
