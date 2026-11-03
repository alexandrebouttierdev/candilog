//! Tests de « Compléter une entreprise » : sources simulées, modèle simulé par un faux Ollama.

use super::*;
use crate::features::referentials::infrastructure::SqliteReferentialRepository;
use crate::features::settings::domain::{AppSettings, SettingsRepository};
use crate::features::settings::infrastructure::SqliteSettingsRepository;
use crate::features::sync::infrastructure::authorized::test_server::{serve, Handler};
use async_trait::async_trait;
use std::sync::Mutex;

const SIRET_RENNES: &str = "38012986612345";

fn candidat(siret: &str, city: &str, head_office: bool, activity: &str) -> CompanyCandidate {
    CompanyCandidate {
        siret: siret.into(),
        siren: siret[..9].into(),
        name: "Nova Conseil".into(),
        trade_name: None,
        city: Some(city.into()),
        postal_code: Some("35000".into()),
        address: Some(format!("4 Rue de la Paix 35000 {city}")),
        head_office,
        activity_code: Some(activity.into()),
        activity_label: naf_section(activity)
            .and_then(naf_section_label)
            .map(str::to_owned),
        legal_category: Some("5710".into()),
        size_category: Some("PME".into()),
        workforce_range: Some("21".into()),
        open_establishments: Some(2),
        source_url: format!("https://annuaire-entreprises.data.gouv.fr/etablissement/{siret}"),
    }
}

struct Annuaire {
    etablissements: Vec<CompanyCandidate>,
    erreur: Option<SourceError>,
}

#[async_trait]
impl CompanyRegistry for Annuaire {
    async fn search(&self, _query: &str) -> Result<Vec<CompanyCandidate>, SourceError> {
        match &self.erreur {
            Some(erreur) => Err(erreur.clone()),
            None => Ok(self.etablissements.clone()),
        }
    }
    async fn establishment(&self, siret: &str) -> Result<Option<CompanyCandidate>, SourceError> {
        Ok(self
            .etablissements
            .iter()
            .find(|c| c.siret == siret)
            .cloned())
    }
}

struct Web(Result<Vec<WebResult>, SourceError>, Mutex<Vec<String>>);

#[async_trait]
impl WebSearch for Web {
    async fn search(
        &self,
        query: &str,
        _preferred: Option<&str>,
    ) -> Result<Vec<WebResult>, SourceError> {
        self.1.lock().unwrap().push(query.to_owned());
        self.0.clone()
    }
}

struct Connaissance(Option<(String, String)>);

#[async_trait]
impl KnowledgeBase for Connaissance {
    async fn official_website(
        &self,
        _siren: &str,
    ) -> Result<Option<(String, String)>, SourceError> {
        Ok(self.0.clone())
    }
}

struct Pages;

#[async_trait]
impl PageReader for Pages {
    async fn summary(&self, url: &str) -> Result<PageSummary, SourceError> {
        Ok(PageSummary {
            url: url.into(),
            title: Some("Nova Conseil — cabinet de conseil".into()),
            description: Some("Conseil en organisation à Rennes.".into()),
        })
    }
}

fn resultats_web() -> Vec<WebResult> {
    vec![
        WebResult {
            title: "NOVA CONSEIL (Rennes) - Société.com".into(),
            url: "https://www.societe.com/societe/nova-conseil-380129866.html".into(),
            snippet: "Chiffre d'affaires, dirigeants…".into(),
        },
        WebResult {
            title: "Nova Conseil — Accueil".into(),
            url: "https://www.nova-conseil.fr/accueil".into(),
            snippet: "Cabinet de conseil en organisation".into(),
        },
    ]
}

struct Banc {
    service: AiService,
    requetes_web: Arc<Web>,
    _dossier: tempfile::TempDir,
}

fn banc(annuaire: Annuaire, web: Result<Vec<WebResult>, SourceError>, site: Option<&str>) -> Banc {
    let pool = crate::core::database::open_pool(None).unwrap();
    crate::core::database::run_local_migrations(&pool).unwrap();
    let dossier = tempfile::tempdir().unwrap();
    let managed = Arc::new(ManagedOllamaService::new(
        Arc::new(SqliteSettingsRepository::new(pool.clone())),
        crate::features::ai::application::ManagedOllamaPaths {
            runtime_root: dossier.path().join("runtime"),
            models_dir: dossier.path().join("models"),
            downloads_dir: dossier.path().join("downloads"),
        },
    ));
    let web = Arc::new(Web(web, Mutex::new(Vec::new())));
    let sources = CompanySources {
        registry: Arc::new(annuaire),
        web: Arc::clone(&web) as Arc<dyn WebSearch>,
        knowledge: Arc::new(Connaissance(site.map(|site| {
            (
                site.to_owned(),
                "http://www.wikidata.org/entity/Q42".to_owned(),
            )
        }))),
        pages: Arc::new(Pages),
    };
    let service = AiService::new(
        pool.clone(),
        Arc::new(
            crate::features::profile::infrastructure::SqliteProfileRepository::new(pool.clone()),
        ),
        managed,
        CompanyResearchDeps {
            sources,
            referentials: Arc::new(SqliteReferentialRepository::new(pool)),
        },
    );
    Banc {
        service,
        requetes_web: web,
        _dossier: dossier,
    }
}

/// Configure un Ollama simulé qui renvoie `reponse` à toute génération.
async fn avec_ollama(banc: &Banc, reponse: serde_json::Value) {
    let handler: Handler = Arc::new(move |_| {
        let corps = serde_json::json!({
            "message": {"content": reponse.to_string()},
            "prompt_eval_count": 100,
            "eval_count": 20
        });
        (200, vec![], serde_json::to_vec(&corps).unwrap())
    });
    let endpoint = serve(handler).await;
    let repo = SqliteSettingsRepository::new(banc.service.pool.clone());
    let settings = AppSettings {
        llm: LlmConfig {
            provider: ProviderKind::Ollama,
            api_key: None,
            endpoint: Some(endpoint),
            model: "modele-test".into(),
            temperature: 0.2,
            mode: AnalysisMode::Auto,
        },
        ..AppSettings::default()
    };
    repo.upsert(&settings).unwrap();
}

fn demande(siret: Option<&str>) -> CompanyResearchRequest {
    CompanyResearchRequest {
        generation_id: "recherche-1".into(),
        name: "Nova Conseil".into(),
        siret: siret.map(str::to_owned),
    }
}

#[tokio::test]
async fn plusieurs_etablissements_sont_proposes_au_choix() {
    let banc = banc(
        Annuaire {
            etablissements: vec![
                candidat("38012986600001", "Paris", true, "70.22Z"),
                candidat(SIRET_RENNES, "Rennes", false, "70.22Z"),
            ],
            erreur: None,
        },
        Ok(vec![]),
        None,
    );
    let resultat = banc
        .service
        .search_companies(
            CompanySearchRequest {
                generation_id: "g".into(),
                name: "Nova Conseil".into(),
                location: None,
            },
            |_| {},
        )
        .await
        .unwrap()
        .output;
    assert_eq!(resultat.candidates.len(), 2);
    assert!(!resultat.confident, "deux villes : l'utilisateur choisit");
}

#[tokio::test]
async fn l_annuaire_sature_ou_injoignable_est_explique() {
    for (erreur, attendu) in [
        (SourceError::RateLimited(Some(3)), "PROVIDER_ERROR"),
        (SourceError::Unavailable, "HTTP_ERROR"),
    ] {
        let banc = banc(
            Annuaire {
                etablissements: vec![],
                erreur: Some(erreur),
            },
            Ok(vec![]),
            None,
        );
        let error = banc
            .service
            .search_companies(
                CompanySearchRequest {
                    generation_id: "g".into(),
                    name: "Nova".into(),
                    location: None,
                },
                |_| {},
            )
            .await
            .unwrap_err();
        assert_eq!(error.code(), attendu);
        if attendu == "PROVIDER_ERROR" {
            assert!(
                error.user_message().contains("3 s"),
                "{}",
                error.user_message()
            );
        }
    }
}

#[tokio::test]
async fn sans_ia_les_faits_de_l_annuaire_et_de_wikidata_suffisent_a_un_resultat_partiel() {
    let banc = banc(
        Annuaire {
            etablissements: vec![candidat(SIRET_RENNES, "Rennes", false, "62.02A")],
            erreur: None,
        },
        Ok(resultats_web()),
        Some("https://www.nova-conseil.fr/"),
    );
    let etapes = Mutex::new(Vec::new());
    let resultat = banc
        .service
        .research_company(demande(Some(SIRET_RENNES)), |p| {
            etapes.lock().unwrap().push(p.step)
        })
        .await
        .unwrap()
        .output;

    let champs = &resultat.fields;
    assert_eq!(champs.city.value.as_deref(), Some("Rennes"));
    assert_eq!(champs.city.origin, Some(FieldOrigin::Registry));
    assert!(champs
        .city
        .note
        .as_deref()
        .unwrap()
        .contains("Établissement local"));
    assert_eq!(champs.company_size.value.as_deref(), Some("PME"));
    // Section J : secteur établi sans le modèle.
    assert_eq!(
        champs.sector_id.value.as_deref(),
        Some("5ec70000-0000-4000-8000-00000000000d")
    );
    assert_eq!(
        champs.website.value.as_deref(),
        Some("https://www.nova-conseil.fr")
    );
    assert_eq!(champs.website.origin, Some(FieldOrigin::Wikidata));
    // ESN, conseil ou éditeur : sans le modèle, le type reste vide plutôt que deviné.
    assert_eq!(champs.company_type_id.value, None);
    assert!(!resultat.ai_used);
    assert!(
        resultat
            .warnings
            .iter()
            .any(|w| w.starts_with("IA indisponible")),
        "{:?}",
        resultat.warnings
    );
    assert!(resultat
        .sources
        .iter()
        .any(|s| s.url.contains("annuaire-entreprises.data.gouv.fr")));
    assert!(etapes.lock().unwrap().iter().any(|e| e == "Recherche Web"));
    assert_eq!(
        banc.requetes_web.1.lock().unwrap().as_slice(),
        ["Nova Conseil Rennes"],
        "la recherche Web précise la ville de l'établissement"
    );
}

#[tokio::test]
async fn le_modele_complete_ce_qui_manque_dans_les_listes_fermees() {
    let banc = banc(
        Annuaire {
            etablissements: vec![candidat(SIRET_RENNES, "Rennes", false, "70.22Z")],
            erreur: None,
        },
        Ok(resultats_web()),
        None,
    );
    let referentiels = banc.service.company_research.referentials.load().unwrap();
    let rang_conseil = referentiels
        .sectors
        .iter()
        .position(|s| s.name.starts_with("Conseil"))
        .unwrap();
    avec_ollama(
        &banc,
        serde_json::json!({
            "secteur": sector_code(rang_conseil),
            "type": "CONSULTING_FIRM",
            "site": "U1"
        }),
    )
    .await;

    let resultat = banc
        .service
        .research_company(demande(Some(SIRET_RENNES)), |_| {})
        .await
        .unwrap()
        .output;

    let champs = &resultat.fields;
    assert!(resultat.ai_used, "{:?}", resultat.warnings);
    assert_eq!(
        champs.sector_id.value,
        Some(referentiels.sectors[rang_conseil].id.to_string())
    );
    assert_eq!(champs.sector_id.origin, Some(FieldOrigin::Ai));
    assert_eq!(
        champs.company_type_id.value.as_deref(),
        Some("CONSULTING_FIRM")
    );
    // U1 est le premier site **non annuaire** : societe.com n'est jamais proposé.
    assert_eq!(
        champs.website.value.as_deref(),
        Some("https://www.nova-conseil.fr")
    );
    assert_eq!(
        champs.website.source_url.as_deref(),
        Some("https://www.nova-conseil.fr/accueil")
    );
    // La ville ne passe jamais par le modèle.
    assert_eq!(champs.city.origin, Some(FieldOrigin::Registry));
}

#[tokio::test]
async fn un_code_invente_par_le_modele_est_ecarte() {
    let banc = banc(
        Annuaire {
            etablissements: vec![candidat(SIRET_RENNES, "Rennes", false, "70.22Z")],
            erreur: None,
        },
        Ok(resultats_web()),
        None,
    );
    avec_ollama(
        &banc,
        serde_json::json!({"secteur": "S99", "type": "LICORNE", "site": "https://invente.example"}),
    )
    .await;

    let champs = banc
        .service
        .research_company(demande(Some(SIRET_RENNES)), |_| {})
        .await
        .unwrap()
        .output
        .fields;
    assert_eq!(champs.sector_id.value, None);
    assert_eq!(champs.company_type_id.value, None);
    assert_eq!(champs.website.value, None);
}

#[tokio::test]
async fn sans_annuaire_ni_web_rien_n_est_invente() {
    let banc = banc(
        Annuaire {
            etablissements: vec![],
            erreur: None,
        },
        Err(SourceError::Unavailable),
        None,
    );
    let resultat = banc
        .service
        .research_company(demande(None), |_| {})
        .await
        .unwrap()
        .output;
    assert_eq!(resultat.fields.city.value, None);
    assert_eq!(resultat.fields.address.value, None);
    assert!(resultat
        .warnings
        .iter()
        .any(|w| w.contains("recherche Web")));
}

#[tokio::test]
async fn un_etablissement_disparu_ou_un_siret_forge_est_refuse() {
    let banc = banc(
        Annuaire {
            etablissements: vec![],
            erreur: None,
        },
        Ok(vec![]),
        None,
    );
    let disparu = banc
        .service
        .research_company(demande(Some(SIRET_RENNES)), |_| {})
        .await
        .unwrap_err();
    assert_eq!(disparu.code(), "NOT_FOUND");
    let forge = banc
        .service
        .research_company(demande(Some("../../etc")), |_| {})
        .await
        .unwrap_err();
    assert_eq!(forge.code(), "VALIDATION_ERROR");
}

#[test]
fn l_invite_n_expose_que_les_listes_utiles_et_encadre_le_web() {
    let invite = invite_classement(
        "Nova",
        None,
        None,
        Some(&[("CONSULTING_FIRM".into(), "Cabinet de conseil".into())]),
        &["https://www.nova.fr".into()],
        None,
        &resultats_web(),
    );
    assert!(!invite.contains("Secteurs possibles"));
    assert!(invite.contains("CONSULTING_FIRM : Cabinet de conseil"));
    assert!(invite.contains("U1 : https://www.nova.fr"));
    assert!(invite.contains(DONNEES_NON_FIABLES));
}
