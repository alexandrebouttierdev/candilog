//! Appels HTTP communs aux sources de recherche d'entreprise.
//!
//! Chaque source est un service public tiers, gratuit et sans clé : Candilog s'y présente
//! par un `User-Agent` descriptif, n'envoie qu'une requête par étape, borne la taille des
//! réponses et respecte un `429` (`Retry-After`) au lieu d'insister.

use crate::features::ai::domain::SourceError;
use serde::de::DeserializeOwned;
use std::time::Duration;

/// Identification demandée par les services publics interrogés.
pub const USER_AGENT: &str = concat!(
    "Candilog/",
    env!("CARGO_PKG_VERSION"),
    " (application de bureau ; +https://candilog.fr)"
);

/// Délai d'une requête : une source lente ne doit pas figer la recherche.
pub const TIMEOUT: Duration = Duration::from_secs(12);

/// Réponse JSON d'une API, au plus.
pub const MAX_JSON_BYTES: usize = 2 * 1024 * 1024;

/// Client des API publiques (annuaire, Wikidata, SearXNG).
///
/// # Errors
/// `Unavailable` si le client TLS ne peut être construit.
pub fn api_client() -> Result<reqwest::Client, SourceError> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(6))
        .timeout(TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
        .map_err(|error| {
            tracing::error!(%error, "client HTTP de recherche indisponible");
            SourceError::Unavailable
        })
}

/// Traduit le statut d'une réponse en échec de source, ou `None` si elle est exploitable.
pub fn status_error(response: &reqwest::Response) -> Option<SourceError> {
    let status = response.status();
    if status.is_success() {
        return None;
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let attente = response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.trim().parse::<u64>().ok());
        return Some(SourceError::RateLimited(attente));
    }
    if status.is_server_error() {
        return Some(SourceError::Unavailable);
    }
    Some(SourceError::Invalid(format!("HTTP {}", status.as_u16())))
}

/// Erreur de transport (DNS, connexion, délai) : la source est indisponible.
pub fn transport_error(error: &reqwest::Error) -> SourceError {
    tracing::warn!(
        is_timeout = error.is_timeout(),
        is_connect = error.is_connect(),
        "source de recherche injoignable"
    );
    SourceError::Unavailable
}

/// Lit le corps d'une réponse sans dépasser `max_bytes`.
///
/// # Errors
/// `Invalid` si le corps dépasse la borne, `Unavailable` si le transfert s'interrompt.
pub async fn read_capped(
    mut response: reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, SourceError> {
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| transport_error(&error))?
    {
        if body.len() + chunk.len() > max_bytes {
            return Err(SourceError::Invalid("réponse trop volumineuse".into()));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

/// `GET` d'une API JSON publique.
///
/// # Errors
/// Échec de transport, statut d'erreur, corps trop volumineux ou JSON illisible.
pub async fn get_json<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: url::Url,
    accept: &str,
) -> Result<T, SourceError> {
    let response = client
        .get(url)
        .header(reqwest::header::ACCEPT, accept)
        .send()
        .await
        .map_err(|error| transport_error(&error))?;
    if let Some(error) = status_error(&response) {
        return Err(error);
    }
    let body = read_capped(response, MAX_JSON_BYTES).await?;
    serde_json::from_slice(&body).map_err(|error| {
        tracing::warn!(%error, "réponse JSON d'une source de recherche illisible");
        SourceError::Invalid("réponse illisible".into())
    })
}
