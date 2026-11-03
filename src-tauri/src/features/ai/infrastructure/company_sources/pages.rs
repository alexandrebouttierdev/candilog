//! Lecture bornée de la page d'accueil d'un site d'entreprise.
//!
//! L'URL vient de Wikidata ou d'un résultat de recherche : c'est une **entrée non fiable**.
//! Elle est donc traitée comme un endpoint IA distant (`docs/AI.md`) : HTTP(S) seulement,
//! adresse résolue puis vérifiée publique et **épinglée** pour la connexion, redirections
//! suivies à la main et revérifiées à chaque saut, lecture plafonnée. On n'en garde que le
//! titre et la description, en texte.

use super::http::{status_error, transport_error, TIMEOUT, USER_AGENT};
use crate::core::utils::validation::is_local_or_private_ip;
use crate::features::ai::domain::{PageReader, PageSummary, SourceError};
use async_trait::async_trait;

/// Octets lus au plus : le `<head>` d'une page d'accueil y tient largement.
const MAX_PAGE_BYTES: usize = 512 * 1024;
const MAX_REDIRECTIONS: usize = 3;
const MAX_DESCRIPTION: usize = 400;
const MAX_TITRE: usize = 200;

/// Lecteur de pages publiques.
pub struct PublicPages {
    /// Autorise la boucle locale : réservé aux tests, qui servent leurs pages en local.
    allow_local: bool,
}

impl PublicPages {
    #[must_use]
    pub const fn new() -> Self {
        Self { allow_local: false }
    }

    #[cfg(test)]
    const fn locales() -> Self {
        Self { allow_local: true }
    }

    async fn client_pour(&self, url: &url::Url) -> Result<reqwest::Client, SourceError> {
        if !matches!(url.scheme(), "http" | "https") {
            return Err(SourceError::Invalid("adresse non Web".into()));
        }
        let host = url
            .host_str()
            .ok_or_else(|| SourceError::Invalid("adresse sans hôte".into()))?
            .to_owned();
        let port = url.port_or_known_default().unwrap_or(443);
        let adresses: Vec<_> = tokio::net::lookup_host((host.as_str(), port))
            .await
            .map_err(|_| SourceError::Unavailable)?
            .collect();
        let Some(adresse) = adresses.first().copied() else {
            return Err(SourceError::Unavailable);
        };
        if !self.allow_local && adresses.iter().any(|a| is_local_or_private_ip(a.ip())) {
            return Err(SourceError::Invalid("adresse locale refusée".into()));
        }
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(std::time::Duration::from_secs(6))
            .timeout(TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .resolve(&host, adresse)
            .build()
            .map_err(|error| transport_error(&error))
    }
}

impl Default for PublicPages {
    fn default() -> Self {
        Self::new()
    }
}

/// Lit au plus `max` octets du corps, sans échouer au-delà : seul le début compte.
async fn debut_du_corps(
    mut response: reqwest::Response,
    max: usize,
) -> Result<Vec<u8>, SourceError> {
    let mut corps = Vec::new();
    while corps.len() < max {
        match response
            .chunk()
            .await
            .map_err(|error| transport_error(&error))?
        {
            Some(morceau) => corps.extend_from_slice(&morceau),
            None => break,
        }
    }
    corps.truncate(max);
    Ok(corps)
}

#[async_trait]
impl PageReader for PublicPages {
    async fn summary(&self, url: &str) -> Result<PageSummary, SourceError> {
        let mut courante =
            url::Url::parse(url).map_err(|_| SourceError::Invalid("adresse illisible".into()))?;
        for _ in 0..=MAX_REDIRECTIONS {
            let client = self.client_pour(&courante).await?;
            let response = client
                .get(courante.clone())
                .header(reqwest::header::ACCEPT, "text/html")
                .send()
                .await
                .map_err(|error| transport_error(&error))?;
            if response.status().is_redirection() {
                let cible = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .and_then(|location| courante.join(location).ok())
                    .ok_or_else(|| SourceError::Invalid("redirection illisible".into()))?;
                courante = cible;
                continue;
            }
            if let Some(error) = status_error(&response) {
                return Err(error);
            }
            let html = response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.to_ascii_lowercase().contains("text/html"));
            if !html {
                return Err(SourceError::Invalid("la page n'est pas du HTML".into()));
            }
            let corps = debut_du_corps(response, MAX_PAGE_BYTES).await?;
            let texte = String::from_utf8_lossy(&corps);
            return Ok(PageSummary {
                url: courante.to_string(),
                title: titre(&texte),
                description: description(&texte),
            });
        }
        Err(SourceError::Invalid("trop de redirections".into()))
    }
}

/// Position insensible à la casse de `aiguille` dans `texte`, à partir de `depuis`.
fn trouve(texte: &str, aiguille: &str, depuis: usize) -> Option<usize> {
    texte
        .get(depuis..)?
        .to_ascii_lowercase()
        .find(aiguille)
        .map(|position| position + depuis)
}

fn nettoie(brut: &str, max: usize) -> Option<String> {
    let texte = brut
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#039;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ");
    let texte: String = texte
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .filter(|caractere| *caractere != '<' && *caractere != '>')
        .take(max)
        .collect();
    (!texte.is_empty()).then_some(texte)
}

fn titre(html: &str) -> Option<String> {
    let debut = trouve(html, "<title", 0)?;
    let ouverture = html.get(debut..)?.find('>')? + debut + 1;
    let fin = trouve(html, "</title", ouverture)?;
    nettoie(html.get(ouverture..fin)?, MAX_TITRE)
}

/// Valeur d'un attribut dans le texte d'une balise.
fn attribut<'a>(balise: &'a str, nom: &str) -> Option<&'a str> {
    let minuscules = balise.to_ascii_lowercase();
    let mut depuis = 0;
    while let Some(position) = minuscules.get(depuis..)?.find(&format!("{nom}=")) {
        let debut = depuis + position;
        // `content=` ne doit pas correspondre à `data-content=`.
        let precedent = minuscules.as_bytes().get(debut.wrapping_sub(1)).copied();
        let valeur_debut = debut + nom.len() + 1;
        if debut == 0 || matches!(precedent, Some(b' ' | b'\t' | b'\n' | b'\r')) {
            let reste = balise.get(valeur_debut..)?;
            let guillemet = reste.chars().next()?;
            if guillemet == '"' || guillemet == '\'' {
                let fin = reste.get(1..)?.find(guillemet)?;
                return reste.get(1..=fin);
            }
            let fin = reste.find([' ', '>', '/']).unwrap_or(reste.len());
            return reste.get(..fin);
        }
        depuis = valeur_debut;
    }
    None
}

fn description(html: &str) -> Option<String> {
    let mut depuis = 0;
    let mut og = None;
    while let Some(debut) = trouve(html, "<meta", depuis) {
        let fin = html
            .get(debut..)?
            .find('>')
            .map_or(html.len(), |f| f + debut);
        let balise = html.get(debut..fin)?;
        let nom = attribut(balise, "name")
            .or_else(|| attribut(balise, "property"))
            .map(str::to_ascii_lowercase);
        match nom.as_deref() {
            Some("description") => {
                if let Some(contenu) =
                    attribut(balise, "content").and_then(|c| nettoie(c, MAX_DESCRIPTION))
                {
                    return Some(contenu);
                }
            }
            Some("og:description") if og.is_none() => {
                og = attribut(balise, "content").and_then(|c| nettoie(c, MAX_DESCRIPTION));
            }
            _ => {}
        }
        depuis = fin.max(debut + 5);
    }
    og
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::sync::infrastructure::authorized::test_server::{serve, Handler};
    use std::sync::Arc;

    const ACCUEIL: &str = r#"<!doctype html><html><head>
        <TITLE>Nova &amp; Cie — Conseil   informatique</TITLE>
        <meta data-content="leurre" property="og:description" content='Version sociale'>
        <meta name="Description" content="ESN rennaise spécialisée en &quot;cloud&quot; et DevOps.">
        </head><body>…</body></html>"#;

    #[test]
    fn titre_et_description_sont_extraits_en_texte() {
        assert_eq!(
            titre(ACCUEIL).as_deref(),
            Some("Nova & Cie — Conseil informatique")
        );
        assert_eq!(
            description(ACCUEIL).as_deref(),
            Some("ESN rennaise spécialisée en \"cloud\" et DevOps.")
        );
        assert_eq!(
            description(r#"<meta property="og:description" content="Seulement OG">"#).as_deref(),
            Some("Seulement OG")
        );
        assert_eq!(titre("<p>sans titre</p>"), None);
    }

    #[tokio::test]
    async fn une_page_est_lue_apres_redirection() {
        let handler: Handler = Arc::new(|request| {
            if request.target == "/" {
                (301, vec![("Location".into(), "/fr/".into())], vec![])
            } else {
                (
                    200,
                    vec![("Content-Type".into(), "text/html; charset=utf-8".into())],
                    ACCUEIL.as_bytes().to_vec(),
                )
            }
        });
        let base = serve(handler).await;
        let page = PublicPages::locales()
            .summary(&format!("{base}/"))
            .await
            .unwrap();
        assert!(page.url.ends_with("/fr/"), "{}", page.url);
        assert!(page.title.is_some());
    }

    #[tokio::test]
    async fn une_adresse_locale_est_refusee_hors_tests() {
        let handler: Handler = Arc::new(|_| panic!("aucune requête attendue"));
        let base = serve(handler).await;
        assert!(matches!(
            PublicPages::new().summary(&base).await,
            Err(SourceError::Invalid(_))
        ));
        assert!(matches!(
            PublicPages::new().summary("file:///etc/passwd").await,
            Err(SourceError::Invalid(_))
        ));
    }

    #[tokio::test]
    async fn un_contenu_qui_n_est_pas_du_html_est_ignore() {
        let handler: Handler = Arc::new(|_| {
            (
                200,
                vec![("Content-Type".into(), "application/pdf".into())],
                b"%PDF".to_vec(),
            )
        });
        let base = serve(handler).await;
        assert!(matches!(
            PublicPages::locales().summary(&base).await,
            Err(SourceError::Invalid(_))
        ));
    }
}
