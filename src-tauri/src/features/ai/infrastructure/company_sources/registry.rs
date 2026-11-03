//! Annuaire officiel des entreprises : API Recherche d'entreprises (data.gouv.fr).
//!
//! Gratuite, sans clé, sous licence ouverte, alimentée par la base SIRENE de l'INSEE. Elle
//! renvoie, pour chaque entreprise, les **établissements** qui correspondent à la
//! recherche — c'est elle qui permet de distinguer Orange à Rennes d'Orange à Paris. Limite
//! annoncée : 7 requêtes par seconde et par adresse ; Candilog en émet une par étape.
//!
//! Schéma : `https://recherche-entreprises.api.gouv.fr/docs/` (`open-api.yml`).

use super::http::{api_client, get_json};
use crate::features::ai::domain::{
    naf_section, naf_section_label, readable_case, CompanyCandidate, CompanyRegistry, SourceError,
};
use async_trait::async_trait;
use serde::Deserialize;

/// Adresse de production de l'API.
pub const API_URL: &str = "https://recherche-entreprises.api.gouv.fr";
/// Fiches publiques des établissements, pour vérification par l'utilisateur.
const FICHE_URL: &str = "https://annuaire-entreprises.data.gouv.fr/etablissement";

/// Entreprises lues par recherche : chacune apporte ses établissements correspondants.
const RESULTATS_PAR_PAGE: &str = "10";
const ETABLISSEMENTS_PAR_ENTREPRISE: &str = "10";

#[derive(Debug, Deserialize)]
struct Payload {
    #[serde(default)]
    results: Vec<Entreprise>,
}

#[derive(Debug, Deserialize)]
struct Entreprise {
    siren: String,
    #[serde(default)]
    nom_complet: Option<String>,
    #[serde(default)]
    nom_raison_sociale: Option<String>,
    #[serde(default)]
    nombre_etablissements_ouverts: Option<u32>,
    #[serde(default)]
    categorie_entreprise: Option<String>,
    #[serde(default)]
    nature_juridique: Option<String>,
    #[serde(default)]
    tranche_effectif_salarie: Option<String>,
    #[serde(default)]
    siege: Option<Etablissement>,
    #[serde(default)]
    matching_etablissements: Vec<Etablissement>,
}

#[derive(Debug, Clone, Deserialize)]
struct Etablissement {
    siret: String,
    #[serde(default)]
    adresse: Option<String>,
    #[serde(default)]
    code_postal: Option<String>,
    #[serde(default)]
    libelle_commune: Option<String>,
    #[serde(default)]
    est_siege: Option<bool>,
    #[serde(default)]
    activite_principale: Option<String>,
    #[serde(default)]
    etat_administratif: Option<String>,
    #[serde(default)]
    liste_enseignes: Option<Vec<String>>,
    #[serde(default)]
    nom_commercial: Option<String>,
}

impl Etablissement {
    /// Établissement fermé (`etat_administratif = "F"`) : jamais proposé.
    fn ouvert(&self) -> bool {
        self.etat_administratif.as_deref() != Some("F")
    }
}

/// Valeur diffusée, hors mentions de non-diffusion (« [NON-DIFFUSIBLE] »).
fn diffusee(value: Option<&str>) -> Option<&str> {
    value
        .map(str::trim)
        .filter(|text| !text.is_empty() && !text.to_ascii_uppercase().contains("NON-DIFFUSIBLE"))
}

fn candidat(entreprise: &Entreprise, etablissement: &Etablissement) -> Option<CompanyCandidate> {
    let siret = etablissement.siret.trim();
    if siret.len() != 14 || !siret.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let name = diffusee(entreprise.nom_raison_sociale.as_deref())
        .or_else(|| diffusee(entreprise.nom_complet.as_deref()))
        .map(readable_case)?;
    let trade_name = etablissement
        .liste_enseignes
        .as_ref()
        .and_then(|enseignes| enseignes.iter().find_map(|e| diffusee(Some(e))))
        .or_else(|| diffusee(etablissement.nom_commercial.as_deref()))
        .map(readable_case)
        .filter(|enseigne| enseigne != &name);
    let activity_code = diffusee(etablissement.activite_principale.as_deref()).map(str::to_owned);
    let activity_label = activity_code
        .as_deref()
        .and_then(naf_section)
        .and_then(naf_section_label)
        .map(str::to_owned);
    Some(CompanyCandidate {
        siret: siret.to_owned(),
        siren: entreprise.siren.trim().to_owned(),
        name,
        trade_name,
        city: diffusee(etablissement.libelle_commune.as_deref()).map(readable_case),
        postal_code: diffusee(etablissement.code_postal.as_deref()).map(str::to_owned),
        address: diffusee(etablissement.adresse.as_deref()).map(readable_case),
        head_office: etablissement.est_siege.unwrap_or(false),
        activity_code,
        activity_label,
        legal_category: diffusee(entreprise.nature_juridique.as_deref()).map(str::to_owned),
        size_category: diffusee(entreprise.categorie_entreprise.as_deref()).map(str::to_owned),
        workforce_range: diffusee(entreprise.tranche_effectif_salarie.as_deref())
            .map(str::to_owned),
        open_establishments: entreprise.nombre_etablissements_ouverts,
        source_url: format!("{FICHE_URL}/{siret}"),
    })
}

/// Établissements ouverts d'une réponse : ceux qui correspondent à la recherche, ou le
/// siège quand l'API n'en signale aucun.
fn candidats(payload: &Payload) -> Vec<CompanyCandidate> {
    payload
        .results
        .iter()
        .flat_map(|entreprise| {
            let etablissements: Vec<&Etablissement> =
                if entreprise.matching_etablissements.is_empty() {
                    entreprise.siege.iter().collect()
                } else {
                    entreprise.matching_etablissements.iter().collect()
                };
            etablissements
                .into_iter()
                .filter(|etablissement| etablissement.ouvert())
                .filter_map(|etablissement| candidat(entreprise, etablissement))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Client de l'API Recherche d'entreprises.
pub struct RechercheEntreprises {
    client: reqwest::Client,
    base: url::Url,
}

impl RechercheEntreprises {
    /// # Errors
    /// `Invalid` si l'adresse de base est illisible, `Unavailable` sans client HTTP.
    pub fn new(base: &str) -> Result<Self, SourceError> {
        Ok(Self {
            client: api_client()?,
            base: url::Url::parse(base)
                .map_err(|_| SourceError::Invalid("adresse de l'annuaire invalide".into()))?,
        })
    }

    async fn query(&self, q: &str) -> Result<Payload, SourceError> {
        let mut url = self
            .base
            .join("/search")
            .map_err(|_| SourceError::Invalid("adresse de l'annuaire invalide".into()))?;
        url.query_pairs_mut()
            .append_pair("q", q)
            .append_pair("page", "1")
            .append_pair("per_page", RESULTATS_PAR_PAGE)
            .append_pair(
                "limite_matching_etablissements",
                ETABLISSEMENTS_PAR_ENTREPRISE,
            )
            .append_pair("etat_administratif", "A");
        get_json(&self.client, url, "application/json").await
    }
}

#[async_trait]
impl CompanyRegistry for RechercheEntreprises {
    async fn search(&self, query: &str) -> Result<Vec<CompanyCandidate>, SourceError> {
        Ok(candidats(&self.query(query).await?))
    }

    async fn establishment(&self, siret: &str) -> Result<Option<CompanyCandidate>, SourceError> {
        // Quatorze chiffres : l'API cherche alors ce SIRET précisément.
        let payload = self.query(siret).await?;
        Ok(payload.results.iter().find_map(|entreprise| {
            entreprise
                .matching_etablissements
                .iter()
                .chain(entreprise.siege.iter())
                .find(|etablissement| etablissement.siret == siret)
                .and_then(|etablissement| candidat(entreprise, etablissement))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::sync::infrastructure::authorized::test_server::{serve, Handler};
    use std::sync::{Arc, Mutex};

    /// Réponse de l'annuaire pour « orange » : un siège et deux établissements locaux,
    /// dont un fermé, plus un homonyme sans établissement correspondant.
    fn reponse_orange() -> serde_json::Value {
        serde_json::json!({
            "results": [
                {
                    "siren": "380129866",
                    "nom_complet": "ORANGE",
                    "nom_raison_sociale": "ORANGE",
                    "nombre_etablissements_ouverts": 1200,
                    "categorie_entreprise": "GE",
                    "nature_juridique": "5599",
                    "tranche_effectif_salarie": "53",
                    "siege": {"siret": "38012986600001", "est_siege": true},
                    "matching_etablissements": [
                        {
                            "siret": "38012986600001",
                            "adresse": "111 QUAI DU PRESIDENT ROOSEVELT 92130 ISSY-LES-MOULINEAUX",
                            "code_postal": "92130",
                            "libelle_commune": "ISSY-LES-MOULINEAUX",
                            "est_siege": true,
                            "activite_principale": "61.10Z",
                            "etat_administratif": "A"
                        },
                        {
                            "siret": "38012986612345",
                            "adresse": "4 RUE DU CLOS COURTEL 35510 CESSON-SEVIGNE",
                            "code_postal": "35510",
                            "libelle_commune": "CESSON-SEVIGNE",
                            "est_siege": false,
                            "activite_principale": "61.10Z",
                            "etat_administratif": "A",
                            "liste_enseignes": ["ORANGE LABS"]
                        },
                        {
                            "siret": "38012986699999",
                            "adresse": "1 RUE FERMEE 69000 LYON",
                            "libelle_commune": "LYON",
                            "etat_administratif": "F"
                        }
                    ]
                },
                {
                    "siren": "123456789",
                    "nom_complet": "ORANGE BLEUE",
                    "nom_raison_sociale": null,
                    "siege": {
                        "siret": "12345678900011",
                        "adresse": "[NON-DIFFUSIBLE]",
                        "libelle_commune": "NANTES",
                        "est_siege": true,
                        "etat_administratif": "A"
                    },
                    "matching_etablissements": []
                }
            ],
            "total_results": 2, "page": 1, "per_page": 10, "total_pages": 1
        })
    }

    #[tokio::test]
    async fn les_etablissements_ouverts_sont_lus_et_normalises() {
        let vus = Arc::new(Mutex::new(Vec::new()));
        let journal = Arc::clone(&vus);
        let handler: Handler = Arc::new(move |request| {
            journal.lock().unwrap().push(request.clone());
            (200, vec![], serde_json::to_vec(&reponse_orange()).unwrap())
        });
        let base = serve(handler).await;
        let registre = RechercheEntreprises::new(&base).unwrap();

        let candidats = registre.search("orange").await.unwrap();

        assert_eq!(
            candidats.len(),
            3,
            "le fermé est écarté, l'homonyme garde son siège"
        );
        let rennes = &candidats[1];
        assert_eq!(rennes.city.as_deref(), Some("Cesson-Sevigne"));
        assert_eq!(rennes.trade_name.as_deref(), Some("Orange Labs"));
        assert!(!rennes.head_office);
        assert_eq!(
            rennes.activity_label.as_deref(),
            Some("Information et communication")
        );
        assert_eq!(
            rennes.source_url,
            "https://annuaire-entreprises.data.gouv.fr/etablissement/38012986612345"
        );
        assert!(candidats[0].head_office);
        assert_eq!(candidats[0].size_category.as_deref(), Some("GE"));
        // Adresse non diffusée : jamais affichée comme une adresse.
        assert_eq!(candidats[2].address, None);
        assert_eq!(candidats[2].name, "Orange Bleue");

        let requete = vus.lock().unwrap()[0].target.clone();
        assert!(requete.starts_with("/search?q=orange&"), "{requete}");
        assert!(requete.contains("etat_administratif=A"));
        assert!(vus.lock().unwrap()[0]
            .header("user-agent")
            .is_some_and(|agent| agent.starts_with("Candilog/")));
    }

    #[tokio::test]
    async fn un_etablissement_est_relu_par_son_siret() {
        let handler: Handler =
            Arc::new(|_| (200, vec![], serde_json::to_vec(&reponse_orange()).unwrap()));
        let registre = RechercheEntreprises::new(&serve(handler).await).unwrap();

        let trouve = registre.establishment("38012986612345").await.unwrap();
        assert_eq!(trouve.unwrap().postal_code.as_deref(), Some("35510"));
        assert_eq!(
            registre.establishment("00000000000000").await.unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn la_limite_de_debit_est_rapportee_avec_son_delai() {
        let handler: Handler = Arc::new(|_| {
            (
                429,
                vec![("Retry-After".into(), "3".into())],
                b"{}".to_vec(),
            )
        });
        let registre = RechercheEntreprises::new(&serve(handler).await).unwrap();
        assert_eq!(
            registre.search("orange").await,
            Err(SourceError::RateLimited(Some(3)))
        );
    }

    #[tokio::test]
    async fn une_panne_ou_une_reponse_illisible_est_signalee() {
        let panne: Handler = Arc::new(|_| (503, vec![], vec![]));
        let registre = RechercheEntreprises::new(&serve(panne).await).unwrap();
        assert_eq!(registre.search("x").await, Err(SourceError::Unavailable));

        let illisible: Handler = Arc::new(|_| (200, vec![], b"<html>".to_vec()));
        let registre = RechercheEntreprises::new(&serve(illisible).await).unwrap();
        assert!(matches!(
            registre.search("x").await,
            Err(SourceError::Invalid(_))
        ));
    }
}
