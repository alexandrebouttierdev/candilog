//! Site officiel d'une entreprise d'après Wikidata, rapproché par son numéro SIREN.
//!
//! La propriété `P1616` (numéro SIREN) relie une entité Wikidata à l'entreprise de
//! l'annuaire sans ambiguïté de nom ; `P856` porte son site officiel. Service public,
//! gratuit, sans clé ; sa politique demande un `User-Agent` identifiable.

use super::http::{api_client, get_json};
use crate::features::ai::domain::{KnowledgeBase, SourceError};
use async_trait::async_trait;
use serde::Deserialize;

/// Point d'accès SPARQL de production.
pub const SPARQL_URL: &str = "https://query.wikidata.org/sparql";

#[derive(Debug, Deserialize)]
struct Reponse {
    results: Resultats,
}

#[derive(Debug, Deserialize)]
struct Resultats {
    #[serde(default)]
    bindings: Vec<Liaison>,
}

#[derive(Debug, Deserialize)]
struct Liaison {
    item: Valeur,
    #[serde(default)]
    site: Option<Valeur>,
}

#[derive(Debug, Deserialize)]
struct Valeur {
    value: String,
}

/// Client du point d'accès SPARQL de Wikidata.
pub struct Wikidata {
    client: reqwest::Client,
    endpoint: url::Url,
}

impl Wikidata {
    /// # Errors
    /// `Invalid` si l'adresse est illisible, `Unavailable` sans client HTTP.
    pub fn new(endpoint: &str) -> Result<Self, SourceError> {
        Ok(Self {
            client: api_client()?,
            endpoint: url::Url::parse(endpoint)
                .map_err(|_| SourceError::Invalid("adresse de Wikidata invalide".into()))?,
        })
    }
}

#[async_trait]
impl KnowledgeBase for Wikidata {
    async fn official_website(&self, siren: &str) -> Result<Option<(String, String)>, SourceError> {
        // Le SIREN est interpolé dans la requête : neuf chiffres, rien d'autre.
        if siren.len() != 9 || !siren.bytes().all(|byte| byte.is_ascii_digit()) {
            return Ok(None);
        }
        let requete = format!(
            "SELECT ?item ?site WHERE {{ ?item wdt:P1616 \"{siren}\" . OPTIONAL {{ ?item wdt:P856 ?site }} }} LIMIT 5"
        );
        let mut url = self.endpoint.clone();
        url.query_pairs_mut()
            .append_pair("query", &requete)
            .append_pair("format", "json");
        let reponse: Reponse =
            get_json(&self.client, url, "application/sparql-results+json").await?;
        Ok(reponse.results.bindings.into_iter().find_map(|liaison| {
            let site = liaison.site?.value;
            matches!(url::Url::parse(&site).ok()?.scheme(), "http" | "https")
                .then_some((site, liaison.item.value))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::sync::infrastructure::authorized::test_server::{serve, Handler};
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn le_site_officiel_est_trouve_par_le_siren() {
        let vus = Arc::new(Mutex::new(Vec::new()));
        let journal = Arc::clone(&vus);
        let handler: Handler = Arc::new(move |request| {
            journal.lock().unwrap().push(request.clone());
            let corps = serde_json::json!({"results": {"bindings": [
                {"item": {"value": "http://www.wikidata.org/entity/Q1431486"}},
                {"item": {"value": "http://www.wikidata.org/entity/Q1431486"},
                 "site": {"value": "https://www.orange.com/"}}
            ]}});
            (200, vec![], serde_json::to_vec(&corps).unwrap())
        });
        let wikidata = Wikidata::new(&format!("{}/sparql", serve(handler).await)).unwrap();

        let trouve = wikidata.official_website("380129866").await.unwrap();
        assert_eq!(
            trouve,
            Some((
                "https://www.orange.com/".into(),
                "http://www.wikidata.org/entity/Q1431486".into()
            ))
        );
        let requete = vus.lock().unwrap()[0].target.clone();
        assert!(requete.contains("P1616"), "{requete}");
        assert!(requete.contains("380129866"));
    }

    #[tokio::test]
    async fn un_siren_mal_forme_n_est_jamais_envoye() {
        let handler: Handler = Arc::new(|_| panic!("aucune requête attendue"));
        let wikidata = Wikidata::new(&format!("{}/sparql", serve(handler).await)).unwrap();
        assert_eq!(
            wikidata.official_website("38012\" } DROP").await.unwrap(),
            None
        );
    }
}
