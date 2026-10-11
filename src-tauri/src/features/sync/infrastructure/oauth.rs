//! OAuth 2.0 pour application de bureau publique (RFC 8252) : code d'autorisation, PKCE
//! S256, redirection sur la boucle locale `127.0.0.1` vers un port éphémère.
//!
//! L'utilisateur s'authentifie dans **son** navigateur, chez Google ou Microsoft : Candilog
//! ne voit jamais son mot de passe. Aucun secret confidentiel n'est embarqué — Microsoft
//! n'en utilise pas pour une application publique, et la valeur que Google fournit aux
//! applications de bureau n'est pas confidentielle par conception (« it is assumed that
//! these apps cannot keep secrets ») : PKCE et le contrôle de `state` portent la sécurité.
//!
//! Les jetons ne sont jamais journalisés. Seul le jeton de renouvellement est conservé, dans
//! le coffre système ; le jeton d'accès vit en mémoire.

use crate::features::sync::domain::remote::RemoteError;
use crate::features::sync::infrastructure::secrets::SyncSecrets;
use base64::Engine;
use ring::rand::{SecureRandom, SystemRandom};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

/// Délai laissé à l'utilisateur pour se connecter dans son navigateur.
const AUTHORIZATION_TIMEOUT: Duration = Duration::from_secs(300);
/// Marge avant expiration à partir de laquelle le jeton d'accès est renouvelé.
const REFRESH_MARGIN: Duration = Duration::from_secs(60);

/// Application OAuth d'un fournisseur.
#[derive(Debug, Clone)]
pub struct OAuthClient {
    /// Identifiant de l'entrée du coffre (`google_drive`, `one_drive`).
    pub provider: &'static str,
    pub client_id: String,
    /// Valeur non confidentielle exigée par certains types de clients Google.
    pub client_secret: Option<String>,
    pub authorize_url: String,
    pub token_url: String,
    pub revoke_url: Option<String>,
    pub scopes: String,
    pub extra_authorize: Vec<(&'static str, &'static str)>,
    /// Microsoft attend la liste des permissions aussi à l'échange du code.
    pub scope_on_token: bool,
}

/// Échec de l'autorisation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OAuthError {
    #[error("autorisation refusée par l'utilisateur")]
    Denied,
    #[error("connexion abandonnée")]
    Cancelled,
    #[error("délai de connexion dépassé")]
    TimedOut,
    /// Retour d'autorisation qui ne correspond pas à la demande (attaque ou onglet périmé).
    #[error("réponse d'autorisation invalide")]
    InvalidCallback,
    /// Session révoquée ou expirée : reconnexion nécessaire.
    #[error("session révoquée")]
    Revoked,
    #[error("hors ligne")]
    Offline,
    #[error("erreur du fournisseur : {0}")]
    Provider(String),
}

impl From<OAuthError> for RemoteError {
    fn from(error: OAuthError) -> Self {
        match error {
            OAuthError::Offline => {
                Self::Offline("connexion au service d'authentification impossible".into())
            }
            OAuthError::Revoked | OAuthError::Denied => {
                Self::Unauthorized("la session du fournisseur a expiré ou a été révoquée".into())
            }
            other => Self::Provider(other.to_string()),
        }
    }
}

/// Jetons obtenus.
pub struct Tokens {
    pub access_token: Zeroizing<String>,
    pub expires_at: Instant,
    pub refresh_token: Option<Zeroizing<String>>,
}

fn base64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn random_token() -> Result<String, OAuthError> {
    let mut bytes = [0_u8; 32];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| OAuthError::Provider("aléa système indisponible".into()))?;
    Ok(base64url(&bytes))
}

/// Défi PKCE S256 d'un vérificateur (RFC 7636 §4.2).
#[must_use]
pub fn pkce_challenge(verifier: &str) -> String {
    base64url(ring::digest::digest(&ring::digest::SHA256, verifier.as_bytes()).as_ref())
}

/// URL d'autorisation à ouvrir dans le navigateur.
#[must_use]
pub fn authorize_url(
    client: &OAuthClient,
    redirect_uri: &str,
    state: &str,
    challenge: &str,
) -> String {
    let mut url = match url::Url::parse(&client.authorize_url) {
        Ok(url) => url,
        Err(_) => return String::new(),
    };
    {
        let mut query = url.query_pairs_mut();
        query
            .append_pair("client_id", &client.client_id)
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", &client.scopes)
            .append_pair("state", state)
            .append_pair("code_challenge", challenge)
            .append_pair("code_challenge_method", "S256");
        for (key, value) in &client.extra_authorize {
            query.append_pair(key, value);
        }
    }
    url.to_string()
}

/// Issue de la lecture d'une requête reçue sur la boucle locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Callback {
    Code(String),
    Error(String),
    /// Requête sans rapport (icône du site…) : on continue d'attendre.
    Ignored,
}

/// Analyse la ligne de requête HTTP du retour d'autorisation et vérifie `state`.
///
/// # Errors
/// [`OAuthError::InvalidCallback`] si `state` ne correspond pas.
pub fn parse_callback(request: &str, expected_state: &str) -> Result<Callback, OAuthError> {
    let Some(line) = request.lines().next() else {
        return Ok(Callback::Ignored);
    };
    let mut parts = line.split_whitespace();
    let (Some("GET"), Some(target)) = (parts.next(), parts.next()) else {
        return Ok(Callback::Ignored);
    };
    let Some((path, query)) = target.split_once('?') else {
        return Ok(Callback::Ignored);
    };
    if path != "/" {
        return Ok(Callback::Ignored);
    }
    let mut code = None;
    let mut error = None;
    let mut state = None;
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "error" => error = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            _ => {}
        }
    }
    if code.is_none() && error.is_none() {
        return Ok(Callback::Ignored);
    }
    // `state` lie le retour à la demande : sans lui, une page tierce pourrait faire
    // adopter à Candilog un code d'autorisation obtenu pour un autre compte.
    let valid = state.as_deref() == Some(expected_state);
    if !valid {
        return Err(OAuthError::InvalidCallback);
    }
    Ok(error.map_or_else(|| Callback::Code(code.unwrap_or_default()), Callback::Error))
}

fn page(title: &str, message: &str) -> String {
    let body = format!(
        "<!doctype html><html lang=\"fr\"><meta charset=\"utf-8\"><title>Candilog</title>\
         <body style=\"font-family:system-ui;margin:4rem auto;max-width:32rem\">\
         <h1 style=\"font-size:1.3rem\">{title}</h1><p>{message}</p></body></html>"
    );
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\
         Cache-Control: no-store\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

const NOT_FOUND: &str = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";

/// Conduit l'autorisation complète : ouvre le navigateur, attend le retour sur la boucle
/// locale, puis échange le code contre des jetons.
///
/// # Errors
/// [`OAuthError`] : refus, abandon, délai, retour invalide, réseau ou fournisseur.
pub async fn authorize(
    client: &OAuthClient,
    http: &reqwest::Client,
    open_browser: &(dyn Fn(&str) -> Result<(), String> + Send + Sync),
    cancel: &CancellationToken,
) -> Result<Tokens, OAuthError> {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .map_err(|error| OAuthError::Provider(format!("port local indisponible : {error}")))?;
    let port = listener
        .local_addr()
        .map_err(|error| OAuthError::Provider(error.to_string()))?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}");
    let verifier = Zeroizing::new(random_token()?);
    let state = random_token()?;
    let url = authorize_url(client, &redirect_uri, &state, &pkce_challenge(&verifier));
    open_browser(&url).map_err(OAuthError::Provider)?;

    let code = tokio::select! {
        () = cancel.cancelled() => return Err(OAuthError::Cancelled),
        () = tokio::time::sleep(AUTHORIZATION_TIMEOUT) => return Err(OAuthError::TimedOut),
        result = wait_for_code(&listener, &state) => result?,
    };

    let mut form = vec![
        ("client_id", client.client_id.clone()),
        ("code", code.to_string()),
        ("code_verifier", verifier.to_string()),
        ("grant_type", "authorization_code".to_owned()),
        ("redirect_uri", redirect_uri),
    ];
    if let Some(secret) = &client.client_secret {
        form.push(("client_secret", secret.clone()));
    }
    if client.scope_on_token {
        form.push(("scope", client.scopes.clone()));
    }
    request_tokens(http, &client.token_url, &form).await
}

async fn wait_for_code(
    listener: &tokio::net::TcpListener,
    state: &str,
) -> Result<Zeroizing<String>, OAuthError> {
    loop {
        let (mut socket, _) = listener
            .accept()
            .await
            .map_err(|error| OAuthError::Provider(error.to_string()))?;
        let mut buffer = vec![0_u8; 8192];
        let mut read = 0;
        while read < buffer.len() {
            let count = match tokio::time::timeout(
                Duration::from_secs(5),
                socket.read(&mut buffer[read..]),
            )
            .await
            {
                Ok(Ok(count)) => count,
                _ => break,
            };
            if count == 0 {
                break;
            }
            read += count;
            if buffer[..read]
                .windows(4)
                .any(|window| window == b"\r\n\r\n")
            {
                break;
            }
        }
        let request = String::from_utf8_lossy(&buffer[..read]).into_owned();
        let outcome = parse_callback(&request, state);
        let response = match &outcome {
            Ok(Callback::Code(_)) => page(
                "Connexion réussie",
                "Vous pouvez fermer cet onglet et revenir à Candilog.",
            ),
            Ok(Callback::Error(_)) => page(
                "Connexion annulée",
                "L'autorisation n'a pas été accordée. Vous pouvez fermer cet onglet.",
            ),
            Ok(Callback::Ignored) => NOT_FOUND.to_owned(),
            Err(_) => page(
                "Réponse refusée",
                "Cette réponse ne correspond pas à la demande de Candilog. Recommencez depuis l'application.",
            ),
        };
        let _ = socket.write_all(response.as_bytes()).await;
        let _ = socket.shutdown().await;
        match outcome? {
            Callback::Code(code) => return Ok(Zeroizing::new(code)),
            Callback::Error(error) if error == "access_denied" => return Err(OAuthError::Denied),
            Callback::Error(error) => {
                tracing::warn!(error, "autorisation OAuth refusée par le fournisseur");
                return Err(OAuthError::Provider(error));
            }
            Callback::Ignored => {}
        }
    }
}

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    expires_in: Option<u64>,
    #[serde(default)]
    refresh_token: Option<String>,
}

#[derive(serde::Deserialize)]
struct TokenError {
    error: String,
}

async fn request_tokens(
    http: &reqwest::Client,
    token_url: &str,
    form: &[(&str, String)],
) -> Result<Tokens, OAuthError> {
    let response = http
        .post(token_url)
        .form(form)
        .send()
        .await
        .map_err(|error| {
            tracing::warn!(
                is_connect = error.is_connect(),
                is_timeout = error.is_timeout(),
                "point de jetons injoignable"
            );
            OAuthError::Offline
        })?;
    let status = response.status();
    let bytes = response.bytes().await.map_err(|_| OAuthError::Offline)?;
    if !status.is_success() {
        let code = serde_json::from_slice::<TokenError>(&bytes)
            .map(|error| error.error)
            .unwrap_or_default();
        // Seul le code d'erreur part au journal : la réponse peut contenir des
        // identifiants de corrélation, jamais utiles à l'utilisateur.
        tracing::warn!(status = status.as_u16(), code, "échange de jeton refusé");
        return Err(match code.as_str() {
            "invalid_grant" | "interaction_required" | "consent_required" => OAuthError::Revoked,
            "access_denied" => OAuthError::Denied,
            _ if status.is_server_error() => OAuthError::Offline,
            _ => OAuthError::Provider(format!("échange de jeton refusé ({})", status.as_u16())),
        });
    }
    let parsed: TokenResponse = serde_json::from_slice(&bytes)
        .map_err(|_| OAuthError::Provider("réponse de jeton illisible".into()))?;
    if parsed.access_token.is_empty() {
        return Err(OAuthError::Provider("jeton d'accès absent".into()));
    }
    Ok(Tokens {
        access_token: Zeroizing::new(parsed.access_token),
        expires_at: Instant::now()
            + Duration::from_secs(parsed.expires_in.unwrap_or(3600).min(86_400)),
        refresh_token: parsed.refresh_token.map(Zeroizing::new),
    })
}

/// Renouvelle le jeton d'accès.
///
/// # Errors
/// [`OAuthError::Revoked`] si le jeton de renouvellement n'est plus valide.
pub async fn refresh(
    client: &OAuthClient,
    http: &reqwest::Client,
    refresh_token: &str,
) -> Result<Tokens, OAuthError> {
    let mut form = vec![
        ("client_id", client.client_id.clone()),
        ("grant_type", "refresh_token".to_owned()),
        ("refresh_token", refresh_token.to_owned()),
    ];
    if let Some(secret) = &client.client_secret {
        form.push(("client_secret", secret.clone()));
    }
    if client.scope_on_token {
        form.push(("scope", client.scopes.clone()));
    }
    request_tokens(http, &client.token_url, &form).await
}

/// Révoque un jeton chez le fournisseur, au mieux : la déconnexion locale ne dépend pas de
/// la réussite de cet appel.
pub async fn revoke(client: &OAuthClient, http: &reqwest::Client, token: &str) {
    let Some(url) = &client.revoke_url else {
        return;
    };
    match http.post(url).form(&[("token", token)]).send().await {
        Ok(response) if response.status().is_success() => {}
        Ok(response) => tracing::warn!(status = response.status().as_u16(), "révocation refusée"),
        Err(_) => tracing::warn!("révocation impossible : fournisseur injoignable"),
    }
}

struct CachedToken {
    access_token: Zeroizing<String>,
    expires_at: Instant,
}

/// Session authentifiée auprès d'un fournisseur : jeton d'accès en mémoire, renouvelé à
/// la demande depuis le jeton conservé dans le coffre.
pub struct OAuthSession {
    client: OAuthClient,
    http: reqwest::Client,
    secrets: std::sync::Arc<dyn SyncSecrets>,
    cached: tokio::sync::Mutex<Option<CachedToken>>,
}

impl OAuthSession {
    #[must_use]
    pub fn new(
        client: OAuthClient,
        http: reqwest::Client,
        secrets: std::sync::Arc<dyn SyncSecrets>,
    ) -> Self {
        Self {
            client,
            http,
            secrets,
            cached: tokio::sync::Mutex::new(None),
        }
    }

    #[must_use]
    pub const fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// Retient les jetons d'une connexion qui vient d'aboutir.
    ///
    /// # Errors
    /// `Unauthorized` si le fournisseur n'a pas délivré de jeton de renouvellement.
    pub async fn adopt(&self, tokens: Tokens) -> Result<(), RemoteError> {
        let refresh = tokens.refresh_token.ok_or_else(|| {
            RemoteError::Unauthorized("aucun jeton de renouvellement délivré".into())
        })?;
        self.secrets
            .store(
                &crate::features::sync::infrastructure::secrets::token_entry(self.client.provider),
                &refresh,
            )
            .map_err(|error| RemoteError::Provider(error.user_message()))?;
        *self.cached.lock().await = Some(CachedToken {
            access_token: tokens.access_token,
            expires_at: tokens.expires_at,
        });
        Ok(())
    }

    /// Oublie le jeton d'accès en mémoire (après un refus 401).
    pub async fn invalidate(&self) {
        *self.cached.lock().await = None;
    }

    /// Jeton d'accès valide, renouvelé si nécessaire.
    ///
    /// # Errors
    /// `Unauthorized` si aucune session n'existe ou si elle a été révoquée.
    pub async fn bearer(&self) -> Result<Zeroizing<String>, RemoteError> {
        let mut cached = self.cached.lock().await;
        if let Some(token) = cached.as_ref() {
            if token.expires_at > Instant::now() + REFRESH_MARGIN {
                return Ok(token.access_token.clone());
            }
        }
        let entry =
            crate::features::sync::infrastructure::secrets::token_entry(self.client.provider);
        let stored = self
            .secrets
            .load(&entry)
            .map_err(|error| RemoteError::Provider(error.user_message()))?
            .ok_or_else(|| RemoteError::Unauthorized("aucune session enregistrée".into()))?;
        let tokens = refresh(&self.client, &self.http, &stored).await?;
        if let Some(rotated) = &tokens.refresh_token {
            // Le fournisseur peut remplacer le jeton de renouvellement : l'ancien est à
            // jeter (RFC 6749 §6).
            self.secrets
                .store(&entry, rotated)
                .map_err(|error| RemoteError::Provider(error.user_message()))?;
        }
        let access = tokens.access_token.clone();
        *cached = Some(CachedToken {
            access_token: tokens.access_token,
            expires_at: tokens.expires_at,
        });
        Ok(access)
    }

    /// Se déconnecte : révoque si possible, puis oublie tout localement.
    pub async fn sign_out(&self) {
        let entry =
            crate::features::sync::infrastructure::secrets::token_entry(self.client.provider);
        if let Ok(Some(token)) = self.secrets.load(&entry) {
            revoke(&self.client, &self.http, &token).await;
        }
        if let Err(error) = self.secrets.delete(&entry) {
            tracing::warn!(%error, "jeton de synchronisation non retiré du coffre");
        }
        self.invalidate().await;
    }
}

/// Client HTTP des fournisseurs : délais bornés et redirections HTTPS uniquement.
/// `reqwest` retire l'en-tête `Authorization` dès qu'une redirection change d'hôte.
///
/// # Errors
/// Erreur de construction du client (TLS indisponible).
pub fn http_client() -> Result<reqwest::Client, RemoteError> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(120))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.url().scheme() != "https" || attempt.previous().len() >= 5 {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(|error| RemoteError::Provider(format!("client HTTP indisponible : {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::sync::infrastructure::secrets::MemorySecrets;
    use std::sync::Arc;

    fn client(token_url: String) -> OAuthClient {
        OAuthClient {
            provider: "test",
            client_id: "client-public".into(),
            client_secret: None,
            authorize_url: "https://auth.example.test/authorize".into(),
            token_url,
            revoke_url: None,
            scopes: "files.appdata".into(),
            extra_authorize: vec![("access_type", "offline")],
            scope_on_token: false,
        }
    }

    /// Point de jetons simulé : répond une fois avec le statut et le corps donnés.
    async fn token_server(status: u16, body: &'static str) -> String {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let mut buffer = vec![0_u8; 16_384];
                let _ = socket.read(&mut buffer).await;
                let response = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = socket.write_all(response.as_bytes()).await;
            }
        });
        format!("http://127.0.0.1:{port}/token")
    }

    #[test]
    fn le_defi_pkce_suit_la_rfc_7636() {
        assert_eq!(
            pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn l_url_d_autorisation_demande_pkce_s256_sans_secret() {
        let url = authorize_url(
            &client(String::new()),
            "http://127.0.0.1:4000",
            "etat",
            "defi",
        );
        assert!(url.starts_with("https://auth.example.test/authorize?"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("code_challenge=defi"));
        assert!(url.contains("state=etat"));
        assert!(url.contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A4000"));
        assert!(!url.contains("secret"));
    }

    #[test]
    fn le_retour_d_autorisation_verifie_state() {
        assert_eq!(
            parse_callback("GET /?code=abc&state=s1 HTTP/1.1\r\n", "s1").unwrap(),
            Callback::Code("abc".into())
        );
        assert_eq!(
            parse_callback("GET /?code=abc&state=autre HTTP/1.1\r\n", "s1"),
            Err(OAuthError::InvalidCallback)
        );
        assert_eq!(
            parse_callback("GET /?code=abc HTTP/1.1\r\n", "s1"),
            Err(OAuthError::InvalidCallback)
        );
        assert_eq!(
            parse_callback("GET /?error=access_denied&state=s1 HTTP/1.1\r\n", "s1").unwrap(),
            Callback::Error("access_denied".into())
        );
        assert_eq!(
            parse_callback("GET /favicon.ico HTTP/1.1\r\n", "s1").unwrap(),
            Callback::Ignored
        );
        assert_eq!(
            parse_callback("GET /autre?code=x&state=s1 HTTP/1.1\r\n", "s1").unwrap(),
            Callback::Ignored
        );
    }

    /// Simule le navigateur : suit l'URL d'autorisation et rappelle la boucle locale.
    fn browser(
        reply: impl Fn(&str) -> String + Send + Sync + 'static,
    ) -> impl Fn(&str) -> Result<(), String> + Send + Sync {
        move |url: &str| {
            let parsed = url::Url::parse(url).unwrap();
            let pairs: std::collections::HashMap<_, _> =
                parsed.query_pairs().into_owned().collect();
            let redirect = pairs["redirect_uri"].clone();
            let query = reply(&pairs["state"]);
            tokio::spawn(async move {
                let address = redirect.trim_start_matches("http://");
                let mut stream = tokio::net::TcpStream::connect(address).await.unwrap();
                stream
                    .write_all(format!("GET /?{query} HTTP/1.1\r\nHost: x\r\n\r\n").as_bytes())
                    .await
                    .unwrap();
                let mut response = String::new();
                let _ = stream.read_to_string(&mut response).await;
            });
            Ok(())
        }
    }

    #[tokio::test]
    async fn une_autorisation_accordee_donne_des_jetons() {
        let url = token_server(
            200,
            r#"{"access_token":"acces","expires_in":3600,"refresh_token":"renouvellement"}"#,
        )
        .await;
        let http = reqwest::Client::new();
        let open = browser(|state| format!("code=le-code&state={state}"));
        let tokens = authorize(&client(url), &http, &open, &CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(tokens.access_token.as_str(), "acces");
        assert_eq!(
            tokens.refresh_token.as_deref().map(String::as_str),
            Some("renouvellement")
        );
    }

    #[tokio::test]
    async fn un_refus_de_l_utilisateur_est_reconnu() {
        let url = token_server(200, "{}").await;
        let http = reqwest::Client::new();
        let open = browser(|state| format!("error=access_denied&state={state}"));
        let result = authorize(&client(url), &http, &open, &CancellationToken::new()).await;
        assert_eq!(result.err(), Some(OAuthError::Denied));
    }

    #[tokio::test]
    async fn un_retour_falsifie_est_refuse() {
        let url = token_server(200, "{}").await;
        let http = reqwest::Client::new();
        let open = browser(|_| "code=vole&state=forge".to_owned());
        let result = authorize(&client(url), &http, &open, &CancellationToken::new()).await;
        assert_eq!(result.err(), Some(OAuthError::InvalidCallback));
    }

    #[tokio::test]
    async fn une_connexion_abandonnee_s_arrete() {
        let http = reqwest::Client::new();
        let cancel = CancellationToken::new();
        cancel.cancel();
        let open = |_: &str| Ok(());
        let result = authorize(
            &client("http://127.0.0.1:9/token".into()),
            &http,
            &open,
            &cancel,
        )
        .await;
        assert_eq!(result.err(), Some(OAuthError::Cancelled));
    }

    #[tokio::test]
    async fn un_jeton_revoque_demande_une_reconnexion() {
        let url = token_server(
            400,
            r#"{"error":"invalid_grant","error_description":"Token has been expired or revoked."}"#,
        )
        .await;
        let secrets = Arc::new(MemorySecrets::default());
        secrets.store("sync-oauth-test", "ancien").unwrap();
        let session = OAuthSession::new(client(url), reqwest::Client::new(), secrets);
        assert!(matches!(
            session.bearer().await,
            Err(RemoteError::Unauthorized(_))
        ));
    }

    #[tokio::test]
    async fn un_jeton_expire_est_renouvele_et_le_nouveau_conserve() {
        let url = token_server(
            200,
            r#"{"access_token":"neuf","expires_in":3600,"refresh_token":"tourne"}"#,
        )
        .await;
        let secrets = Arc::new(MemorySecrets::default());
        secrets.store("sync-oauth-test", "ancien").unwrap();
        let session = OAuthSession::new(client(url), reqwest::Client::new(), secrets.clone());
        assert_eq!(session.bearer().await.unwrap().as_str(), "neuf");
        assert_eq!(
            secrets.load("sync-oauth-test").unwrap().unwrap().as_str(),
            "tourne"
        );
    }

    #[tokio::test]
    async fn sans_session_enregistree_il_faut_se_connecter() {
        let session = OAuthSession::new(
            client("http://127.0.0.1:9".into()),
            reqwest::Client::new(),
            Arc::new(MemorySecrets::default()),
        );
        assert!(matches!(
            session.bearer().await,
            Err(RemoteError::Unauthorized(_))
        ));
    }
}
