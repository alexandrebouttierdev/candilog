//! Requêtes authentifiées vers un fournisseur cloud, et classement de leurs échecs.

use crate::features::sync::domain::remote::RemoteError;
use crate::features::sync::infrastructure::oauth::OAuthSession;
use std::time::Duration;

/// Corps maximal accepté d'une réponse : au-delà, l'objet n'est pas un objet Candilog.
const MAX_RESPONSE_BYTES: usize = crate::features::sync::domain::protocol::MAX_OBJECT_BYTES;

fn network(error: &reqwest::Error) -> RemoteError {
    tracing::warn!(
        is_connect = error.is_connect(),
        is_timeout = error.is_timeout(),
        "fournisseur de synchronisation injoignable"
    );
    if error.is_connect() || error.is_timeout() || error.is_request() {
        RemoteError::Offline("fournisseur injoignable".into())
    } else {
        RemoteError::Provider("réponse réseau inattendue".into())
    }
}

fn retry_after(response: &reqwest::Response) -> Option<Duration> {
    response
        .headers()
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
        .map(|seconds| Duration::from_secs(seconds.min(3600)))
}

/// Envoie une requête avec le jeton d'accès ; un refus 401 provoque un seul renouvellement
/// du jeton puis une seule nouvelle tentative. Les statuts 404 et 409 sont rendus à
/// l'appelant, qui sait les interpréter.
///
/// # Errors
/// [`RemoteError`] classé : hors ligne, limite de débit, session révoquée, accès refusé.
pub async fn send(
    session: &OAuthSession,
    build: impl Fn(&reqwest::Client) -> reqwest::RequestBuilder,
) -> Result<reqwest::Response, RemoteError> {
    for attempt in 0..2 {
        let token = session.bearer().await?;
        let response = build(session.http())
            .bearer_auth(token.as_str())
            .send()
            .await
            .map_err(|error| network(&error))?;
        let status = response.status().as_u16();
        if status == 401 && attempt == 0 {
            session.invalidate().await;
            continue;
        }
        return classify(response).await;
    }
    Err(RemoteError::Unauthorized(
        "session refusée par le fournisseur".into(),
    ))
}

async fn classify(response: reqwest::Response) -> Result<reqwest::Response, RemoteError> {
    let status = response.status().as_u16();
    match status {
        200..=299 | 404 | 409 => Ok(response),
        401 => Err(RemoteError::Unauthorized(
            "session refusée par le fournisseur".into(),
        )),
        429 | 503 => Err(RemoteError::RateLimited(retry_after(&response))),
        403 | 507 => {
            let delay = retry_after(&response);
            let body = response.text().await.unwrap_or_default();
            if body.contains("rateLimitExceeded") || body.contains("userRateLimitExceeded") {
                Err(RemoteError::RateLimited(delay))
            } else if body.contains("storageQuotaExceeded")
                || body.contains("quotaLimitReached")
                || status == 507
            {
                Err(RemoteError::Unavailable(
                    "espace de stockage du fournisseur plein".into(),
                ))
            } else {
                tracing::warn!(status, "accès refusé par le fournisseur");
                Err(RemoteError::Unavailable(
                    "accès refusé par le fournisseur".into(),
                ))
            }
        }
        500..=599 => Err(RemoteError::Provider(format!(
            "erreur du fournisseur ({status})"
        ))),
        _ => {
            tracing::warn!(status, "réponse inattendue du fournisseur");
            Err(RemoteError::Provider(format!(
                "réponse inattendue ({status})"
            )))
        }
    }
}

/// Corps d'une réponse, borné.
///
/// # Errors
/// `Provider` si le corps dépasse la limite ou ne se lit pas.
pub async fn body(response: reqwest::Response) -> Result<Vec<u8>, RemoteError> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(RemoteError::Provider(
            "objet distant trop volumineux".into(),
        ));
    }
    let bytes = response.bytes().await.map_err(|error| network(&error))?;
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err(RemoteError::Provider(
            "objet distant trop volumineux".into(),
        ));
    }
    Ok(bytes.to_vec())
}

/// Corps JSON d'une réponse.
///
/// # Errors
/// `Provider` si la réponse n'est pas le JSON attendu.
pub async fn json<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, RemoteError> {
    let bytes = body(response).await?;
    serde_json::from_slice(&bytes)
        .map_err(|_| RemoteError::Provider("réponse illisible du fournisseur".into()))
}

/// Un nom d'objet ne contient que des segments simples : rien qui puisse changer le sens
/// d'une URL ou d'une requête de recherche.
#[must_use]
pub fn valid_object_name(name: &str) -> bool {
    !name.is_empty()
        && name.split('/').all(|segment| {
            !segment.is_empty()
                && !segment.starts_with('.')
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
        })
}

#[cfg(test)]
pub mod test_server {
    //! Serveur HTTP minimal pour simuler un fournisseur dans les tests.

    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// Requête reçue par le serveur simulé.
    #[derive(Debug, Clone)]
    pub struct Request {
        pub method: String,
        pub target: String,
        pub headers: Vec<(String, String)>,
        pub body: Vec<u8>,
    }

    impl Request {
        pub fn header(&self, name: &str) -> Option<&str> {
            self.headers
                .iter()
                .find(|(key, _)| key.eq_ignore_ascii_case(name))
                .map(|(_, value)| value.as_str())
        }
    }

    pub type Handler = Arc<dyn Fn(&Request) -> (u16, Vec<(String, String)>, Vec<u8>) + Send + Sync>;

    /// Démarre le serveur et renvoie son adresse de base (`http://127.0.0.1:port`).
    pub async fn serve(handler: Handler) -> String {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let handler = Arc::clone(&handler);
                tokio::spawn(async move {
                    let mut data = Vec::new();
                    let mut buffer = [0_u8; 8192];
                    let header_end = loop {
                        let count = socket.read(&mut buffer).await.unwrap_or(0);
                        if count == 0 {
                            return;
                        }
                        data.extend_from_slice(&buffer[..count]);
                        if let Some(position) = data.windows(4).position(|w| w == b"\r\n\r\n") {
                            break position + 4;
                        }
                    };
                    let head = String::from_utf8_lossy(&data[..header_end]).into_owned();
                    let mut lines = head.split("\r\n");
                    let mut first = lines.next().unwrap_or_default().split_whitespace();
                    let method = first.next().unwrap_or_default().to_owned();
                    let target = first.next().unwrap_or_default().to_owned();
                    let headers: Vec<(String, String)> = lines
                        .filter_map(|line| line.split_once(':'))
                        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
                        .collect();
                    let length: usize = headers
                        .iter()
                        .find(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                        .and_then(|(_, value)| value.parse().ok())
                        .unwrap_or(0);
                    while data.len() < header_end + length {
                        let count = socket.read(&mut buffer).await.unwrap_or(0);
                        if count == 0 {
                            break;
                        }
                        data.extend_from_slice(&buffer[..count]);
                    }
                    let request = Request {
                        method,
                        target,
                        headers,
                        body: data[header_end..].to_vec(),
                    };
                    let (status, extra, body) = handler(&request);
                    let mut response = format!(
                        "HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n",
                        body.len()
                    );
                    for (key, value) in extra {
                        response.push_str(&format!("{key}: {value}\r\n"));
                    }
                    response.push_str("\r\n");
                    let mut bytes = response.into_bytes();
                    bytes.extend_from_slice(&body);
                    let _ = socket.write_all(&bytes).await;
                });
            }
        });
        format!("http://127.0.0.1:{port}")
    }

    /// Point de jetons qui délivre toujours un jeton d'accès.
    pub fn token_ok() -> (u16, Vec<(String, String)>, Vec<u8>) {
        (
            200,
            vec![("Content-Type".into(), "application/json".into())],
            br#"{"access_token":"acces-test","expires_in":3600}"#.to_vec(),
        )
    }
}
