//! Recherche Web par SearXNG, métamoteur libre, sans clé ni compte.
//!
//! Ordre d'essai : l'instance que l'utilisateur a choisie (la sienne, ou une instance
//! publique de confiance), puis des instances publiques tirées de l'annuaire `searx.space`.
//! Aucune n'est supposée disponible : beaucoup désactivent le format JSON ou limitent les
//! requêtes automatiques. Une instance qui refuse (`403`, `429`) est laissée tranquille et
//! la suivante est essayée, trois au plus ; la recherche continue ensuite sans résultats
//! Web plutôt que d'insister.

use super::http::{api_client, get_json};
use crate::features::ai::domain::{SourceError, WebResult, WebSearch};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Annuaire public des instances SearXNG. Son format peut changer : il est lu
/// défensivement, et ignoré s'il devient illisible.
pub const DIRECTORY_URL: &str = "https://searx.space/data/instances.json";
/// Instances essayées au plus par recherche.
const MAX_ESSAIS: usize = 3;
/// Résultats conservés par recherche.
const MAX_RESULTATS: usize = 8;
/// Durée de validité de l'annuaire en mémoire.
const DUREE_ANNUAIRE: Duration = Duration::from_secs(6 * 3600);
/// Longueurs gardées d'un résultat.
const MAX_TITRE: usize = 200;
const MAX_EXTRAIT: usize = 400;

#[derive(Debug, Deserialize)]
struct Reponse {
    #[serde(default)]
    results: Vec<Resultat>,
}

#[derive(Debug, Deserialize)]
struct Resultat {
    #[serde(default)]
    title: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    content: String,
}

fn tronque(texte: &str, max: usize) -> String {
    texte
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(max)
        .collect()
}

/// Instances utilisables d'après l'annuaire : réseau ordinaire (ni Tor ni I2P), en HTTPS,
/// sans erreur relevée. Les clés absentes ne disqualifient pas une instance — le format de
/// l'annuaire n'est pas garanti —, une erreur explicite si.
fn instances_de_l_annuaire(annuaire: &serde_json::Value) -> Vec<String> {
    let Some(instances) = annuaire
        .get("instances")
        .and_then(serde_json::Value::as_object)
    else {
        return Vec::new();
    };
    let mut retenues: Vec<(f64, String)> = instances
        .iter()
        .filter(|(url, _)| url.starts_with("https://"))
        .filter(|(_, info)| {
            info.get("network_type")
                .and_then(serde_json::Value::as_str)
                .is_none_or(|reseau| reseau == "normal")
        })
        .filter(|(_, info)| {
            let http = info.get("http");
            let statut_ok = http
                .and_then(|http| http.get("status_code"))
                .and_then(serde_json::Value::as_u64)
                .is_none_or(|code| code == 200);
            let sans_erreur = http
                .and_then(|http| http.get("error"))
                .is_none_or(serde_json::Value::is_null);
            statut_ok && sans_erreur
        })
        .map(|(url, info)| {
            let reussite = info
                .pointer("/timing/search/success_percentage")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0);
            (reussite, url.trim_end_matches('/').to_owned())
        })
        .collect();
    retenues.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    retenues.into_iter().map(|(_, url)| url).collect()
}

/// Client SearXNG, avec annuaire d'instances en mémoire.
pub struct Searxng {
    client: reqwest::Client,
    directory: Option<url::Url>,
    cache: Mutex<Option<(Instant, Vec<String>)>>,
}

impl Searxng {
    /// `directory` : annuaire des instances publiques, `None` pour n'utiliser que l'instance
    /// choisie par l'utilisateur.
    ///
    /// # Errors
    /// `Unavailable` sans client HTTP.
    pub fn new(directory: Option<&str>) -> Result<Self, SourceError> {
        Ok(Self {
            client: api_client()?,
            directory: directory.and_then(|url| url::Url::parse(url).ok()),
            cache: Mutex::new(None),
        })
    }

    async fn instances_publiques(&self) -> Vec<String> {
        if let Ok(cache) = self.cache.lock() {
            if let Some((lu_a, instances)) = cache.as_ref() {
                if lu_a.elapsed() < DUREE_ANNUAIRE {
                    return instances.clone();
                }
            }
        }
        let Some(directory) = self.directory.clone() else {
            return Vec::new();
        };
        let instances = match get_json::<serde_json::Value>(
            &self.client,
            directory,
            "application/json",
        )
        .await
        {
            Ok(annuaire) => instances_de_l_annuaire(&annuaire),
            Err(error) => {
                tracing::warn!(?error, "annuaire des instances SearXNG indisponible");
                Vec::new()
            }
        };
        if let Ok(mut cache) = self.cache.lock() {
            *cache = Some((Instant::now(), instances.clone()));
        }
        instances
    }

    async fn interroger(&self, instance: &str, query: &str) -> Result<Vec<WebResult>, SourceError> {
        // Une instance peut être servie sous un chemin (`https://exemple.fr/searx`) : la
        // recherche est `<instance>/search`, quelle que soit la barre oblique finale.
        let mut url = url::Url::parse(&format!("{}/search", instance.trim_end_matches('/')))
            .map_err(|_| SourceError::Invalid("adresse SearXNG invalide".into()))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(SourceError::Invalid("adresse SearXNG invalide".into()));
        }
        url.query_pairs_mut()
            .append_pair("q", query)
            .append_pair("format", "json")
            .append_pair("language", "fr")
            .append_pair("safesearch", "1");
        let reponse: Reponse = get_json(&self.client, url, "application/json").await?;
        Ok(reponse
            .results
            .into_iter()
            .filter(|resultat| {
                url::Url::parse(&resultat.url)
                    .is_ok_and(|url| matches!(url.scheme(), "http" | "https"))
            })
            .take(MAX_RESULTATS)
            .map(|resultat| WebResult {
                title: tronque(&resultat.title, MAX_TITRE),
                url: resultat.url,
                snippet: tronque(&resultat.content, MAX_EXTRAIT),
            })
            .collect())
    }
}

#[async_trait]
impl WebSearch for Searxng {
    async fn search(
        &self,
        query: &str,
        preferred: Option<&str>,
    ) -> Result<Vec<WebResult>, SourceError> {
        let mut instances: Vec<String> = preferred.map(str::to_owned).into_iter().collect();
        for publique in self.instances_publiques().await {
            if !instances.contains(&publique) {
                instances.push(publique);
            }
        }
        if instances.is_empty() {
            return Err(SourceError::Unavailable);
        }
        let mut derniere = SourceError::Unavailable;
        for instance in instances.iter().take(MAX_ESSAIS) {
            match self.interroger(instance, query).await {
                Ok(resultats) => return Ok(resultats),
                Err(error) => {
                    tracing::info!(?error, "instance SearXNG écartée pour cette recherche");
                    derniere = error;
                }
            }
        }
        Err(derniere)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::sync::infrastructure::authorized::test_server::{serve, Handler};
    use std::sync::Arc;

    fn resultats() -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({"results": [
            {"title": "Orange  - Opérateur", "url": "https://www.orange.fr/", "content": "Le site officiel   d'Orange"},
            {"title": "Script", "url": "javascript:alert(1)", "content": ""}
        ]}))
        .unwrap()
    }

    #[test]
    fn l_annuaire_ne_garde_que_les_instances_saines() {
        let annuaire = serde_json::json!({"instances": {
            "https://lente.example/": {"network_type": "normal", "http": {"status_code": 200, "error": null},
                "timing": {"search": {"success_percentage": 40.0}}},
            "https://fiable.example/": {"network_type": "normal", "http": {"status_code": 200},
                "timing": {"search": {"success_percentage": 99.0}}},
            "https://panne.example/": {"http": {"status_code": 502}},
            "https://erreur.example/": {"http": {"error": "Timeout"}},
            "http://enclair.example/": {},
            "https://tor.example/": {"network_type": "tor"}
        }});
        assert_eq!(
            instances_de_l_annuaire(&annuaire),
            vec!["https://fiable.example", "https://lente.example"]
        );
        assert!(instances_de_l_annuaire(&serde_json::json!({"format": "nouveau"})).is_empty());
    }

    #[tokio::test]
    async fn l_instance_choisie_est_interrogee_en_json() {
        let handler: Handler = Arc::new(|request| {
            assert!(request
                .target
                .starts_with("/search?q=orange+rennes&format=json"));
            (200, vec![], resultats())
        });
        let base = serve(handler).await;
        let searx = Searxng::new(None).unwrap();

        let trouves = searx.search("orange rennes", Some(&base)).await.unwrap();
        assert_eq!(trouves.len(), 1, "un lien non Web est écarté");
        assert_eq!(trouves[0].title, "Orange - Opérateur");
        assert_eq!(trouves[0].snippet, "Le site officiel d'Orange");
    }

    #[tokio::test]
    async fn une_instance_qui_refuse_le_json_laisse_place_a_la_suivante() {
        let refuse: Handler = Arc::new(|_| (403, vec![], b"Forbidden".to_vec()));
        let accepte: Handler = Arc::new(|_| (200, vec![], resultats()));
        let premiere = serve(refuse).await;
        let seconde = serve(accepte).await;
        // Le serveur de test est en HTTP, que l'annuaire écarterait : l'instance publique
        // est donc placée directement dans le cache de l'annuaire.
        let searx = Searxng::new(None).unwrap();
        *searx.cache.lock().unwrap() = Some((Instant::now(), vec![seconde.clone()]));

        let trouves = searx.search("orange", Some(&premiere)).await.unwrap();
        assert_eq!(trouves.len(), 1);
    }

    #[tokio::test]
    async fn sans_instance_joignable_la_recherche_web_est_indisponible() {
        let sature: Handler = Arc::new(|_| (429, vec![], vec![]));
        let searx = Searxng::new(None).unwrap();
        assert_eq!(
            searx.search("orange", Some(&serve(sature).await)).await,
            Err(SourceError::RateLimited(None))
        );
        assert_eq!(
            Searxng::new(None).unwrap().search("orange", None).await,
            Err(SourceError::Unavailable)
        );
    }
}
