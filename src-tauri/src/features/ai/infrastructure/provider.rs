//! Adaptateurs HTTP des fournisseurs supportés par les réglages historiques.

use crate::core::errors::{AppError, AppResult};
use crate::core::utils::validation::is_local_or_private_ip;
use crate::features::ai::domain::{LlmConfig, ProviderKind};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::sync::Arc;

const MAX_RESPONSE: usize = 5 * 1024 * 1024;

/// Tentatives d'un appel au fournisseur, reprise comprise.
///
/// Une génération de CV enchaîne trois appels et dure une à deux minutes : un incident
/// réseau passager sur le dernier annulait tout le travail et laissait payés les deux
/// appels déjà aboutis. Trois tentatives couvrent la coupure passagère sans transformer une
/// panne durable en attente interminable.
const TENTATIVES: u32 = 3;

/// Attente avant la reprise numéro `numero` (1 pour la première) : 1 s, puis 2 s.
fn attente(numero: u32) -> std::time::Duration {
    std::time::Duration::from_millis(500u64 << numero.min(4))
}

/// Un échec transitoire mérite une reprise ; une erreur de configuration, jamais.
///
/// Côté transport, seuls le délai dépassé et la connexion impossible sont repris : ce sont
/// les deux cas que l'application sait déjà nommer, et les seuls dont on puisse dire que
/// l'appel n'a rien produit. Côté serveur, `429` et les `5xx` sont transitoires par
/// définition. Une clé refusée, un modèle inconnu ou une requête malformée renvoient un
/// `4xx` : les retenter coûterait sans rien changer.
fn est_transitoire(error: &reqwest::Error) -> bool {
    error.status().map_or_else(
        || error.is_timeout() || error.is_connect(),
        |status| status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error(),
    )
}

/// Encode un nom de modèle destiné à un **segment de chemin** d'URL.
///
/// Le nom vient des réglages : interpolé tel quel, un `?`, un `#` ou un `..` déplaçait la
/// requête ailleurs sur l'hôte du fournisseur. L'hôte reste épinglé par `Client::resolve`,
/// il n'y avait donc pas de fuite ; l'appel partait simplement au mauvais endroit, avec un
/// message d'erreur incompréhensible.
fn segment_url(valeur: &str) -> String {
    valeur
        .chars()
        .map(|caractere| match caractere {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' | ':' | '@' => {
                caractere.to_string()
            }
            autre => {
                let mut tampon = [0_u8; 4];
                autre
                    .encode_utf8(&mut tampon)
                    .bytes()
                    .map(|octet| format!("%{octet:02X}"))
                    .collect()
            }
        })
        .collect()
}

/// Texte produit par un appel, et le nombre de tokens qu'il a consommés si le fournisseur
/// le rapporte.
///
/// `tokens` vaut `None` pour un fournisseur qui ne renvoie aucune métrique d'usage, afin de
/// ne pas présenter une absence de donnée comme une génération gratuite.
#[derive(Debug)]
pub struct GenerationOutput {
    pub text: String,
    pub tokens: Option<u32>,
}

#[async_trait]
pub trait LlmGenerator: Send + Sync {
    async fn generate(&self, prompt: &str, system: &str, json: bool)
        -> AppResult<GenerationOutput>;
    async fn test(&self) -> AppResult<()>;
    async fn list_models(&self) -> AppResult<Vec<String>>;
}

pub async fn build_provider(config: &LlmConfig) -> AppResult<Arc<dyn LlmGenerator>> {
    let endpoint = config.endpoint_effectif().trim_end_matches('/').to_owned();
    let url = url::Url::parse(&endpoint)
        .map_err(|_| AppError::Validation("Endpoint IA invalide".into()))?;
    let host = url
        .host_str()
        .ok_or_else(|| AppError::Validation("Endpoint IA sans hôte".into()))?;
    let port = url.port_or_known_default().unwrap_or(443);
    let mut builder = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(8))
        .timeout(std::time::Duration::from_secs(60 * 30))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(concat!("Candilog/", env!("CARGO_PKG_VERSION")));
    if !matches!(config.provider, ProviderKind::Ollama) {
        if url.scheme() != "https" {
            return Err(AppError::Validation(
                "Un endpoint IA distant doit utiliser HTTPS".into(),
            ));
        }
        let adresses: Vec<_> = tokio::net::lookup_host((host, port))
            .await
            .map_err(|_| AppError::Validation("Impossible de résoudre l'endpoint IA".into()))?
            .collect();
        if adresses.is_empty() || adresses.iter().any(|a| is_local_or_private_ip(a.ip())) {
            return Err(AppError::Validation(
                "L'endpoint IA distant ne peut pas cibler le réseau local".into(),
            ));
        }
        builder = builder.resolve(host, adresses[0]);
    }
    let client = builder.build().map_err(AppError::from)?;
    Ok(Arc::new(ProviderHttp {
        config: config.clone(),
        endpoint,
        client,
    }))
}

struct ProviderHttp {
    config: LlmConfig,
    endpoint: String,
    client: reqwest::Client,
}

#[async_trait]
impl LlmGenerator for ProviderHttp {
    async fn generate(
        &self,
        prompt: &str,
        system: &str,
        json: bool,
    ) -> AppResult<GenerationOutput> {
        match self.config.provider {
            ProviderKind::Ollama => self.ollama(prompt, system, json).await,
            ProviderKind::Claude => self.claude(prompt, system).await,
            ProviderKind::Gemini => self.gemini(prompt, system, json).await,
            ProviderKind::OpenAI
            | ProviderKind::Mistral
            | ProviderKind::Nvidia
            | ProviderKind::Custom(_) => self.openai(prompt, system, json).await,
        }
    }

    async fn test(&self) -> AppResult<()> {
        self.list_models().await.map(|_| ())
    }

    async fn list_models(&self) -> AppResult<Vec<String>> {
        match self.config.provider {
            ProviderKind::Ollama => self.models_ollama().await,
            ProviderKind::Claude => self.models_claude().await,
            ProviderKind::Gemini => self.models_gemini().await,
            ProviderKind::OpenAI
            | ProviderKind::Mistral
            | ProviderKind::Nvidia
            | ProviderKind::Custom(_) => self.models_openai().await,
        }
    }
}

impl ProviderHttp {
    /// Traduit un échec de transport en erreur destinée à l'utilisateur.
    ///
    /// Ollama tourne sur la machine de l'utilisateur : « Vérifiez votre réseau », le
    /// message générique des erreurs HTTP, envoyait chercher une panne inexistante. Sur une
    /// installation neuve, Ollama est le fournisseur par défaut et n'est le plus souvent
    /// pas encore installé — c'est le premier message d'erreur que voit un nouvel
    /// utilisateur, et il doit nommer la vraie cause.
    fn traduire(&self, error: reqwest::Error) -> AppError {
        if matches!(self.config.provider, ProviderKind::Ollama)
            && (error.is_connect() || error.is_timeout())
        {
            return AppError::Provider(format!(
                "Ollama ne répond pas sur {}. Démarrez-le, ou choisissez un autre fournisseur dans Réglages → Intelligence artificielle.",
                self.endpoint
            ));
        }
        AppError::from(error)
    }

    /// Envoie une requête et reprend les échecs transitoires.
    ///
    /// La requête est reconstruite à chaque tentative : un corps déjà consommé ne se renvoie
    /// pas. L'attente entre deux essais laisse passer une coupure courte et une limite de
    /// débit sans marteler le fournisseur.
    ///
    /// La reprise vit ici et non dans l'orchestration : c'est l'adaptateur qui connaît le
    /// transport, et tous les appels — analyse d'offre, génération, lettre, import de profil,
    /// liste des modèles — en bénéficient de la même façon.
    ///
    /// L'annulation reste immédiate : `AiService` abandonne le futur qui porte cette boucle,
    /// attente comprise.
    async fn envoyer(
        &self,
        construire: impl Fn() -> reqwest::RequestBuilder,
    ) -> AppResult<reqwest::Response> {
        let mut tentative = 0;
        loop {
            let resultat = construire()
                .send()
                .await
                .and_then(reqwest::Response::error_for_status);
            let error = match resultat {
                Ok(response) => return Ok(response),
                Err(error) => error,
            };
            tentative += 1;
            if tentative >= TENTATIVES || !est_transitoire(&error) {
                return Err(self.traduire(error));
            }
            // L'URL et la clé ne sont jamais journalisées : seul le motif l'est.
            tracing::warn!(
                tentative,
                is_timeout = error.is_timeout(),
                is_connect = error.is_connect(),
                status = ?error.status().map(|value| value.as_u16()),
                "appel au fournisseur IA repris"
            );
            tokio::time::sleep(attente(tentative)).await;
        }
    }

    async fn ollama(&self, prompt: &str, system: &str, json: bool) -> AppResult<GenerationOutput> {
        let body = serde_json::json!({"model":self.config.model,"messages":[{"role":"system","content":system},{"role":"user","content":prompt}],"stream":false,"format":if json { serde_json::json!("json") } else { serde_json::Value::Null },"options":{"temperature":self.config.temperature}});
        let response = self
            .envoyer(|| {
                self.client
                    .post(format!("{}/api/chat", self.endpoint))
                    .json(&body)
            })
            .await?;
        let value: serde_json::Value = json_limite(response).await?;
        let text = text(&value, "/message/content")?;
        Ok(GenerationOutput {
            text,
            tokens: total_tokens(&value),
        })
    }

    async fn openai(&self, prompt: &str, system: &str, json: bool) -> AppResult<GenerationOutput> {
        let mut body = serde_json::json!({"model":self.config.model,"messages":[{"role":"system","content":system},{"role":"user","content":prompt}],"temperature":self.config.temperature});
        if json {
            body["response_format"] = serde_json::json!({"type":"json_object"});
        }
        let response = self
            .envoyer(|| {
                self.client
                    .post(format!("{}/v1/chat/completions", self.endpoint))
                    .bearer_auth(self.config.api_key.as_deref().unwrap_or_default())
                    .json(&body)
            })
            .await?;
        let value: serde_json::Value = json_limite(response).await?;
        let text = text(&value, "/choices/0/message/content")?;
        Ok(GenerationOutput {
            text,
            tokens: total_tokens(&value),
        })
    }

    async fn claude(&self, prompt: &str, system: &str) -> AppResult<GenerationOutput> {
        let body = serde_json::json!({"model":self.config.model,"max_tokens":4096,"system":system,"messages":[{"role":"user","content":prompt}],"temperature":self.config.temperature});
        let response = self
            .envoyer(|| {
                self.client
                    .post(format!("{}/v1/messages", self.endpoint))
                    .header(
                        "x-api-key",
                        self.config.api_key.as_deref().unwrap_or_default(),
                    )
                    .header("anthropic-version", "2023-06-01")
                    .json(&body)
            })
            .await?;
        let value: serde_json::Value = json_limite(response).await?;
        let text = text(&value, "/content/0/text")?;
        Ok(GenerationOutput {
            text,
            tokens: total_tokens(&value),
        })
    }

    async fn gemini(&self, prompt: &str, system: &str, json: bool) -> AppResult<GenerationOutput> {
        let body = serde_json::json!({"contents":[{"parts":[{"text":prompt}]}],"systemInstruction":{"parts":[{"text":system}]},"generationConfig":{"temperature":self.config.temperature,"responseMimeType":if json {"application/json"} else {"text/plain"}}});
        let response = self
            .envoyer(|| {
                self.client
                    .post(format!(
                        "{}/v1beta/models/{}:generateContent",
                        self.endpoint,
                        segment_url(&self.config.model)
                    ))
                    .header(
                        "x-goog-api-key",
                        self.config.api_key.as_deref().unwrap_or_default(),
                    )
                    .json(&body)
            })
            .await?;
        let value: serde_json::Value = json_limite(response).await?;
        let text = text(&value, "/candidates/0/content/parts/0/text")?;
        Ok(GenerationOutput {
            text,
            tokens: total_tokens(&value),
        })
    }

    async fn models_ollama(&self) -> AppResult<Vec<String>> {
        let response = self
            .envoyer(|| self.client.get(format!("{}/api/tags", self.endpoint)))
            .await?;
        let value: serde_json::Value = json_limite(response).await?;
        Ok(value
            .pointer("/models")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|model| model.get("name")?.as_str().map(str::to_owned))
            .collect())
    }

    async fn models_openai(&self) -> AppResult<Vec<String>> {
        let response = self
            .envoyer(|| {
                self.client
                    .get(format!("{}/v1/models", self.endpoint))
                    .bearer_auth(self.config.api_key.as_deref().unwrap_or_default())
            })
            .await?;
        let value: serde_json::Value = json_limite(response).await?;
        Ok(value
            .pointer("/data")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|model| model.get("id")?.as_str().map(str::to_owned))
            .collect())
    }

    async fn models_claude(&self) -> AppResult<Vec<String>> {
        let response = self
            .envoyer(|| {
                self.client
                    .get(format!("{}/v1/models", self.endpoint))
                    .header(
                        "x-api-key",
                        self.config.api_key.as_deref().unwrap_or_default(),
                    )
                    .header("anthropic-version", "2023-06-01")
            })
            .await?;
        let value: serde_json::Value = json_limite(response).await?;
        Ok(value
            .pointer("/data")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|model| model.get("id")?.as_str().map(str::to_owned))
            .collect())
    }

    async fn models_gemini(&self) -> AppResult<Vec<String>> {
        let response = self
            .envoyer(|| {
                self.client
                    .get(format!("{}/v1beta/models", self.endpoint))
                    .header(
                        "x-goog-api-key",
                        self.config.api_key.as_deref().unwrap_or_default(),
                    )
            })
            .await?;
        let value: serde_json::Value = json_limite(response).await?;
        Ok(value
            .pointer("/models")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .filter_map(|model| {
                let name = model.get("name")?.as_str()?;
                Some(name.strip_prefix("models/").unwrap_or(name).to_owned())
            })
            .collect())
    }
}

fn text(value: &serde_json::Value, pointer: &str) -> AppResult<String> {
    value
        .pointer(pointer)
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| AppError::Provider("Le fournisseur IA a renvoyé une réponse vide".into()))
}

/// Tokens consommés par un appel, si le fournisseur les rapporte.
///
/// Chaque fournisseur place ce total à un endroit différent — OpenAI et compatibles sous
/// `usage.total_tokens`, Claude sous deux compteurs séparés sans total, Gemini sous
/// `usageMetadata`, Ollama à la racine — et un point de terminaison personnalisé peut
/// n'en rapporter aucun. Best-effort volontaire : l'absence de métrique ne doit jamais faire
/// échouer une génération par ailleurs réussie, elle se traduit par `None`.
fn total_tokens(value: &serde_json::Value) -> Option<u32> {
    let u64_at = |pointer: &str| value.pointer(pointer).and_then(serde_json::Value::as_u64);
    let bounded = |value: u64| value.min(u64::from(u32::MAX)) as u32;

    if let Some(total) = u64_at("/usage/total_tokens") {
        return Some(bounded(total));
    }
    if let Some(total) = u64_at("/usageMetadata/totalTokenCount") {
        return Some(bounded(total));
    }
    let claude_input = u64_at("/usage/input_tokens");
    let claude_output = u64_at("/usage/output_tokens");
    if claude_input.is_some() || claude_output.is_some() {
        return Some(bounded(
            claude_input
                .unwrap_or(0)
                .saturating_add(claude_output.unwrap_or(0)),
        ));
    }
    let ollama_prompt = u64_at("/prompt_eval_count");
    let ollama_eval = u64_at("/eval_count");
    if ollama_prompt.is_some() || ollama_eval.is_some() {
        return Some(bounded(
            ollama_prompt
                .unwrap_or(0)
                .saturating_add(ollama_eval.unwrap_or(0)),
        ));
    }
    None
}

async fn json_limite<T: DeserializeOwned>(mut response: reqwest::Response) -> AppResult<T> {
    if response
        .content_length()
        .is_some_and(|n| n > MAX_RESPONSE as u64)
    {
        return Err(AppError::Provider("Réponse IA trop volumineuse".into()));
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if body.len().saturating_add(chunk.len()) > MAX_RESPONSE {
            return Err(AppError::Provider("Réponse IA trop volumineuse".into()));
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body).map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::ai::domain::AnalysisMode;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// Serveur HTTP minimal : il renvoie les statuts demandés dans l'ordre, puis `200`, et
    /// compte les requêtes reçues. Assez pour observer une reprise sans ajouter de
    /// dépendance de test au projet.
    async fn serveur(statuts: Vec<u16>) -> (String, Arc<AtomicUsize>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let recues = Arc::new(AtomicUsize::new(0));
        let compteur = Arc::clone(&recues);
        tokio::spawn(async move {
            loop {
                let Ok((mut flux, _)) = listener.accept().await else {
                    return;
                };
                let index = compteur.fetch_add(1, Ordering::SeqCst);
                lire_requete(&mut flux).await;
                let statut = statuts.get(index).copied().unwrap_or(200);
                let corps = r#"{"message":{"content":"{}"},"models":[]}"#;
                let reponse = format!(
                    "HTTP/1.1 {statut} S\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{corps}",
                    corps.len()
                );
                let _ = flux.write_all(reponse.as_bytes()).await;
                let _ = flux.shutdown().await;
            }
        });
        (format!("http://127.0.0.1:{port}"), recues)
    }

    /// Draine la requête avant de répondre : un corps laissé en attente casse la connexion
    /// côté client et masquerait le statut qu'on veut observer.
    async fn lire_requete(flux: &mut tokio::net::TcpStream) {
        let mut tampon = Vec::new();
        let mut morceau = [0u8; 1024];
        loop {
            let Ok(lu) = flux.read(&mut morceau).await else {
                return;
            };
            if lu == 0 {
                return;
            }
            tampon.extend_from_slice(&morceau[..lu]);
            let texte = String::from_utf8_lossy(&tampon).into_owned();
            let Some(fin) = texte.find("\r\n\r\n") else {
                continue;
            };
            let taille: usize = texte
                .lines()
                .find_map(|ligne| {
                    let (nom, valeur) = ligne.split_once(':')?;
                    nom.eq_ignore_ascii_case("content-length")
                        .then(|| valeur.trim().parse().ok())?
                })
                .unwrap_or(0);
            if tampon.len() >= fin + 4 + taille {
                return;
            }
        }
    }

    async fn provider(endpoint: &str) -> Arc<dyn LlmGenerator> {
        build_provider(&LlmConfig {
            provider: ProviderKind::Ollama,
            api_key: None,
            endpoint: Some(endpoint.to_owned()),
            model: "modele-de-test".into(),
            temperature: 0.0,
            mode: AnalysisMode::default(),
        })
        .await
        .unwrap()
    }

    /// Une génération de CV enchaîne trois appels : un `503` passager sur le dernier
    /// annulait toute la génération et laissait payés les deux appels déjà aboutis.
    #[tokio::test]
    async fn un_echec_serveur_passager_est_repris() {
        let (endpoint, recues) = serveur(vec![503]).await;

        let sortie = provider(&endpoint)
            .await
            .generate("prompt", "system", true)
            .await
            .unwrap();

        assert_eq!(sortie.text, "{}");
        assert_eq!(
            recues.load(Ordering::SeqCst),
            2,
            "la requête n'a pas été reprise"
        );
    }

    /// Une limite de débit se lève d'elle-même : elle mérite d'attendre, pas d'échouer.
    #[tokio::test]
    async fn une_limite_de_debit_est_reprise() {
        let (endpoint, recues) = serveur(vec![429]).await;

        assert!(provider(&endpoint)
            .await
            .generate("prompt", "system", true)
            .await
            .is_ok());
        assert_eq!(recues.load(Ordering::SeqCst), 2);
    }

    /// Une clé refusée ou un modèle inconnu ne s'arrangeront pas : les retenter ne ferait
    /// que retarder le message que l'utilisateur doit lire.
    #[tokio::test]
    async fn une_erreur_de_configuration_n_est_pas_reprise() {
        let (endpoint, recues) = serveur(vec![401, 401, 401]).await;

        assert!(provider(&endpoint)
            .await
            .generate("prompt", "system", true)
            .await
            .is_err());
        assert_eq!(recues.load(Ordering::SeqCst), 1, "un 4xx a été repris");
    }

    /// Une panne durable s'arrête : la reprise est bornée, elle ne boucle pas.
    #[tokio::test]
    async fn une_panne_durable_s_arrete_apres_le_nombre_de_tentatives() {
        let (endpoint, recues) = serveur(vec![503, 503, 503, 503, 503]).await;

        assert!(provider(&endpoint)
            .await
            .generate("prompt", "system", true)
            .await
            .is_err());
        assert_eq!(recues.load(Ordering::SeqCst), TENTATIVES as usize);
    }

    /// Ollama tourne en local : un message parlant de réseau envoie chercher une panne qui
    /// n'existe pas. C'est le fournisseur par défaut, donc le premier écueil d'une
    /// installation neuve où Ollama n'est pas encore installé.
    #[tokio::test]
    async fn ollama_injoignable_nomme_ollama_et_pas_le_reseau() {
        // Port fermé : la connexion échoue immédiatement, sans attendre de délai.
        let endpoint = "http://127.0.0.1:1".to_owned();
        let erreur = provider(&endpoint)
            .await
            .generate("prompt", "system", true)
            .await
            .expect_err("la connexion doit échouer");

        let AppError::Provider(message) = erreur else {
            panic!("une panne d'Ollama doit être une erreur de fournisseur, pas HTTP");
        };
        assert!(message.contains("Ollama"), "message obtenu : {message}");
        assert!(message.contains("Réglages"), "message obtenu : {message}");
    }

    /// Le nom du modèle vient des réglages et atterrit dans le chemin de l'URL Gemini.
    #[test]
    fn un_nom_de_modele_ne_peut_pas_deplacer_la_requete() {
        assert_eq!(segment_url("gemini-2.5-flash"), "gemini-2.5-flash");
        assert_eq!(segment_url("../../v1/autre"), "..%2F..%2Fv1%2Fautre");
        assert_eq!(segment_url("modele?cle=1#x"), "modele%3Fcle%3D1%23x");
    }

    #[test]
    fn total_tokens_lit_le_format_openai_et_compatibles() {
        let reponse = serde_json::json!({"usage":{"prompt_tokens":120,"completion_tokens":45,"total_tokens":165}});
        assert_eq!(total_tokens(&reponse), Some(165));
    }

    #[test]
    fn total_tokens_additionne_les_deux_compteurs_claude() {
        let reponse = serde_json::json!({"usage":{"input_tokens":300,"output_tokens":80}});
        assert_eq!(total_tokens(&reponse), Some(380));
    }

    #[test]
    fn total_tokens_lit_le_format_gemini() {
        let reponse = serde_json::json!({"usageMetadata":{"promptTokenCount":50,"candidatesTokenCount":20,"totalTokenCount":70}});
        assert_eq!(total_tokens(&reponse), Some(70));
    }

    #[test]
    fn total_tokens_additionne_les_deux_compteurs_ollama() {
        let reponse = serde_json::json!({"prompt_eval_count":40,"eval_count":15});
        assert_eq!(total_tokens(&reponse), Some(55));
    }

    #[test]
    fn total_tokens_distingue_absence_et_zero_rapporte() {
        let reponse = serde_json::json!({"choices":[{"message":{"content":"Bonjour"}}]});
        assert_eq!(total_tokens(&reponse), None);
        assert_eq!(
            total_tokens(&serde_json::json!({"usage":{"total_tokens":0}})),
            Some(0)
        );
    }
}
