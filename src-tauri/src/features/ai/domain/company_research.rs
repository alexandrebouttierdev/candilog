//! Recherche d'une entreprise pour compléter sa fiche.
//!
//! Le principe est celui de tout Candilog : **rien d'inventé**. Ville, adresse, siège et
//! taille ne viennent que de l'annuaire officiel des entreprises ; le site officiel, de
//! Wikidata (par le numéro SIREN) ou d'un résultat de recherche que le modèle désigne
//! parmi une liste fermée ; secteur et type, d'une correspondance déterministe avec le code
//! d'activité ou la nature juridique, et à défaut d'un choix du modèle **dans les
//! référentiels**. Tout ce que le modèle renvoie hors de ces listes est écarté.
//!
//! Le domaine décrit les contrats des sources (`CompanyRegistry`, `WebSearch`,
//! `KnowledgeBase`, `PageReader`) ; leurs implémentations HTTP vivent dans
//! `infrastructure/company_sources`, et peuvent être remplacées ou complétées sans toucher
//! au service.

use crate::core::errors::{AppError, AppResult};
use crate::core::utils::text::search_key;
use crate::features::ai::domain::ValidateAiOutput;
use crate::features::companies::domain::CompanySize;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Longueur maximale du nom recherché.
pub const MAX_COMPANY_QUERY_CHARS: usize = 120;
/// Longueur maximale de la précision de lieu.
pub const MAX_LOCATION_CHARS: usize = 80;
/// Établissements présentés au plus : au-delà, mieux vaut préciser la ville.
pub const MAX_CANDIDATES: usize = 25;

// ---------------------------------------------------------------------------------------
// Contrat IPC
// ---------------------------------------------------------------------------------------

/// Recherche des établissements d'une entreprise d'après son nom.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct CompanySearchRequest {
    pub generation_id: String,
    pub name: String,
    /// Ville ou code postal, pour départager des homonymes.
    pub location: Option<String>,
}

/// Établissement trouvé dans l'annuaire officiel, présenté au choix de l'utilisateur.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct CompanyCandidate {
    /// SIRET de l'établissement : l'identifiant que la seconde étape relit dans l'annuaire.
    pub siret: String,
    pub siren: String,
    /// Dénomination de l'entreprise (unité légale), normalisée.
    pub name: String,
    /// Enseigne ou nom commercial de l'établissement, s'il en a un.
    pub trade_name: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    /// Adresse complète, normalisée ; `None` si l'annuaire ne la diffuse pas.
    pub address: Option<String>,
    /// Siège social de l'entreprise, et non un établissement local.
    pub head_office: bool,
    /// Code d'activité (NAF) de l'établissement, et son libellé de section.
    pub activity_code: Option<String>,
    pub activity_label: Option<String>,
    /// Nature juridique de l'entreprise (code INSEE à quatre chiffres).
    pub legal_category: Option<String>,
    /// Catégorie d'entreprise de l'INSEE : `PME`, `ETI` ou `GE`.
    pub size_category: Option<String>,
    /// Tranche d'effectif salarié de l'entreprise (code INSEE).
    pub workforce_range: Option<String>,
    /// Nombre d'établissements ouverts de l'entreprise.
    pub open_establishments: Option<u32>,
    /// Fiche publique de l'établissement, pour vérification.
    pub source_url: String,
}

/// Résultat de la recherche d'établissements.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct CompanySearchResult {
    pub candidates: Vec<CompanyCandidate>,
    /// Un seul établissement, au nom identique à la recherche : la sélection peut être
    /// sautée, la vérification reste montrée avant tout remplissage.
    pub confident: bool,
    /// Établissements trouvés au-delà de ceux présentés.
    pub truncated: bool,
}

/// Complète les informations d'un établissement choisi, ou d'une entreprise absente de
/// l'annuaire (`siret = None` : recherche Web seule).
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct CompanyResearchRequest {
    pub generation_id: String,
    pub name: String,
    pub siret: Option<String>,
}

/// D'où vient la valeur proposée pour un champ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum FieldOrigin {
    /// Annuaire officiel des entreprises (données SIRENE).
    Registry,
    /// Wikidata, rapproché par le numéro SIREN.
    Wikidata,
    /// Résultat de recherche Web.
    WebSearch,
    /// Choix du modèle, restreint aux référentiels ou aux URL trouvées.
    Ai,
}

/// Valeur proposée pour un champ du formulaire, avec sa provenance.
#[derive(Debug, Clone, Default, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ResearchedField {
    /// `None` : aucune information fiable trouvée.
    pub value: Option<String>,
    pub origin: Option<FieldOrigin>,
    /// Page qui justifie la valeur.
    pub source_url: Option<String>,
    /// Précision affichée sous la valeur (« d'après le code NAF 62.02A »).
    pub note: Option<String>,
}

/// Valeurs proposées pour les champs du formulaire entreprise.
#[derive(Debug, Clone, Default, PartialEq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct CompanyResearchFields {
    pub name: ResearchedField,
    /// Identifiant d'un secteur du référentiel.
    pub sector_id: ResearchedField,
    /// Code d'un type d'entreprise du référentiel.
    pub company_type_id: ResearchedField,
    /// Valeur de `CompanySize` (`PME`, `ETI`…).
    pub company_size: ResearchedField,
    pub website: ResearchedField,
    pub city: ResearchedField,
    pub address: ResearchedField,
}

/// Page consultée pendant la recherche.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ResearchSource {
    pub label: String,
    pub url: String,
}

/// Informations trouvées, à vérifier avant tout remplissage.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct CompanyResearchResult {
    pub fields: CompanyResearchFields,
    pub sources: Vec<ResearchSource>,
    /// Ce qui n'a pas pu être consulté (service indisponible, IA en erreur…) : le résultat
    /// est alors partiel, et le dit.
    pub warnings: Vec<String>,
    /// Le modèle a été consulté.
    pub ai_used: bool,
}

// ---------------------------------------------------------------------------------------
// Contrats des sources
// ---------------------------------------------------------------------------------------

/// Échec d'une source, traduit ensuite en message pour l'utilisateur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceError {
    /// Réseau indisponible, délai dépassé, erreur serveur.
    Unavailable,
    /// Limite de débit atteinte ; délai d'attente annoncé, en secondes.
    RateLimited(Option<u64>),
    /// Réponse illisible ou refusée par la source.
    Invalid(String),
}

/// Résultat de recherche Web.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct WebResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Résumé d'une page Web, borné.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PageSummary {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

/// Annuaire des entreprises (SIRENE).
#[async_trait]
pub trait CompanyRegistry: Send + Sync {
    /// Établissements dont l'entreprise ou l'adresse correspondent à la recherche.
    async fn search(&self, query: &str) -> Result<Vec<CompanyCandidate>, SourceError>;
    /// Établissement désigné par son SIRET.
    async fn establishment(&self, siret: &str) -> Result<Option<CompanyCandidate>, SourceError>;
}

/// Moteur de recherche Web.
#[async_trait]
pub trait WebSearch: Send + Sync {
    /// `preferred` : instance choisie par l'utilisateur, essayée en premier.
    async fn search(
        &self,
        query: &str,
        preferred: Option<&str>,
    ) -> Result<Vec<WebResult>, SourceError>;
}

/// Base de connaissances structurée (Wikidata).
#[async_trait]
pub trait KnowledgeBase: Send + Sync {
    /// Site officiel de l'entreprise de ce SIREN, et la page qui l'atteste.
    async fn official_website(&self, siren: &str) -> Result<Option<(String, String)>, SourceError>;
}

/// Lecture bornée d'une page Web publique.
#[async_trait]
pub trait PageReader: Send + Sync {
    async fn summary(&self, url: &str) -> Result<PageSummary, SourceError>;
}

/// Sources d'une recherche d'entreprise, injectées par `app/state.rs`.
pub struct CompanySources {
    pub registry: std::sync::Arc<dyn CompanyRegistry>,
    pub web: std::sync::Arc<dyn WebSearch>,
    pub knowledge: std::sync::Arc<dyn KnowledgeBase>,
    pub pages: std::sync::Arc<dyn PageReader>,
}

// ---------------------------------------------------------------------------------------
// Validation des entrées
// ---------------------------------------------------------------------------------------

/// Nom recherché, borné et non vide.
///
/// # Errors
/// `Validation` si le nom est vide ou trop long.
pub fn validate_company_query(name: &str) -> AppResult<String> {
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if name.is_empty() {
        return Err(AppError::Validation(
            "Saisissez le nom de l'entreprise à rechercher.".into(),
        ));
    }
    if name.chars().count() > MAX_COMPANY_QUERY_CHARS {
        return Err(AppError::Validation(
            "Le nom de l'entreprise est trop long pour une recherche.".into(),
        ));
    }
    Ok(name)
}

/// Précision de lieu, bornée ; vide devient `None`.
///
/// # Errors
/// `Validation` si la précision est trop longue.
pub fn validate_location(location: Option<&str>) -> AppResult<Option<String>> {
    let Some(location) = location
        .map(|value| value.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };
    if location.chars().count() > MAX_LOCATION_CHARS {
        return Err(AppError::Validation(
            "La ville ou le code postal est trop long.".into(),
        ));
    }
    Ok(Some(location))
}

/// SIRET : quatorze chiffres, rien d'autre ne part vers l'annuaire.
///
/// # Errors
/// `Validation` si la valeur n'a pas la forme d'un SIRET.
pub fn validate_siret(siret: &str) -> AppResult<&str> {
    if siret.len() == 14 && siret.bytes().all(|byte| byte.is_ascii_digit()) {
        Ok(siret)
    } else {
        Err(AppError::Validation(
            "L'établissement choisi n'a pas un SIRET valide.".into(),
        ))
    }
}

// ---------------------------------------------------------------------------------------
// Correspondances déterministes
// ---------------------------------------------------------------------------------------

/// Section de la nomenclature d'activités (NAF) d'un code `62.02A`.
#[must_use]
pub fn naf_section(code: &str) -> Option<char> {
    let division: u8 = code.get(0..2)?.parse().ok()?;
    Some(match division {
        1..=3 => 'A',
        5..=9 => 'B',
        10..=33 => 'C',
        35 => 'D',
        36..=39 => 'E',
        41..=43 => 'F',
        45..=47 => 'G',
        49..=53 => 'H',
        55 | 56 => 'I',
        58..=63 => 'J',
        64..=66 => 'K',
        68 => 'L',
        69..=75 => 'M',
        77..=82 => 'N',
        84 => 'O',
        85 => 'P',
        86..=88 => 'Q',
        90..=93 => 'R',
        94..=96 => 'S',
        97 | 98 => 'T',
        99 => 'U',
        _ => return None,
    })
}

/// Libellé INSEE d'une section NAF.
#[must_use]
pub fn naf_section_label(section: char) -> Option<&'static str> {
    Some(match section {
        'A' => "Agriculture, sylviculture et pêche",
        'B' => "Industries extractives",
        'C' => "Industrie manufacturière",
        'D' => "Production et distribution d'électricité, de gaz et de vapeur",
        'E' => "Eau, assainissement, gestion des déchets et dépollution",
        'F' => "Construction",
        'G' => "Commerce ; réparation d'automobiles et de motocycles",
        'H' => "Transports et entreposage",
        'I' => "Hébergement et restauration",
        'J' => "Information et communication",
        'K' => "Activités financières et d'assurance",
        'L' => "Activités immobilières",
        'M' => "Activités spécialisées, scientifiques et techniques",
        'N' => "Activités de services administratifs et de soutien",
        'O' => "Administration publique",
        'P' => "Enseignement",
        'Q' => "Santé humaine et action sociale",
        'R' => "Arts, spectacles et activités récréatives",
        'S' => "Autres activités de services",
        'T' => "Activités des ménages en tant qu'employeurs",
        'U' => "Activités extra-territoriales",
        _ => return None,
    })
}

/// Préfixe fixe des identifiants de secteurs semés par `init_schema.sql`.
const SECTOR_PREFIX: &str = "5ec70000-0000-4000-8000-0000000000";

/// Secteur du référentiel qu'un code d'activité désigne sans ambiguïté.
///
/// Les sections M, N, O, P, S, T et U regroupent des métiers trop variés (conseil,
/// sécurité, administration, enseignement…) : elles restent au jugement du modèle.
#[must_use]
pub fn sector_from_activity(code: &str) -> Option<String> {
    let division: u8 = code.get(0..2)?.parse().ok()?;
    let suffix = match (naf_section(code)?, division) {
        (_, 90) => "14",
        (_, 93) => "15",
        ('A', _) => "09",
        ('B' | 'C' | 'D' | 'E', _) => "0c",
        ('F', _) => "04",
        ('G', _) => "05",
        ('H', _) => "16",
        ('I', _) => "0a",
        ('J', _) => "0d",
        ('K', _) => "03",
        ('L', _) => "0b",
        ('Q', _) => "11",
        _ => return None,
    };
    Some(format!("{SECTOR_PREFIX}{suffix}"))
}

/// Type d'entreprise qu'établissent la nature juridique ou un code d'activité précis.
///
/// La nature juridique prime : une association reste une association quelle que soit son
/// activité. Une société commerciale de conseil informatique peut être une ESN, un cabinet
/// de conseil ou un éditeur : c'est au modèle d'en juger, d'après son site.
#[must_use]
pub fn company_type_from_registry(
    legal_category: Option<&str>,
    activity_code: Option<&str>,
) -> Option<&'static str> {
    if let Some(legal) = legal_category {
        let by_legal = match legal.get(0..2) {
            Some("92") => Some("ASSOCIATION"),
            Some("71") => Some("PUBLIC_ADMINISTRATION"),
            Some("72") => Some("LOCAL_AUTHORITY"),
            Some("73" | "74") => Some("PUBLIC_INSTITUTION"),
            Some("41") => Some("PUBLIC_COMPANY"),
            _ if legal == "9300" => Some("FOUNDATION"),
            _ => None,
        };
        if by_legal.is_some() {
            return by_legal;
        }
    }
    let code = activity_code?;
    let division: u8 = code.get(0..2)?.parse().ok()?;
    Some(match (division, code) {
        (_, "78.10Z") => "RECRUITMENT_AGENCY",
        (_, "78.20Z") => "TEMP_AGENCY",
        (_, "86.10Z") => "HOSPITAL",
        (_, "85.42Z") => "UNIVERSITY",
        (64, c) if c.starts_with("64.1") => "BANK",
        (65, _) => "INSURANCE_COMPANY",
        (61, _) => "TELECOM_COMPANY",
        (58, c) if c.starts_with("58.2") => "SOFTWARE_PUBLISHER",
        (1..=3, _) => "AGRICULTURAL_COMPANY",
        (10..=33, _) => "INDUSTRIAL_COMPANY",
        (47, _) => "RETAIL_COMPANY",
        (49..=53, _) => "TRANSPORT_COMPANY",
        (68, _) => "REAL_ESTATE_COMPANY",
        (85, _) => "EDUCATIONAL_INSTITUTION",
        (86, _) => "HEALTHCARE_INSTITUTION",
        _ => return None,
    })
}

/// Tranches d'effectif de l'INSEE sous dix salariés (y compris « non employeuse »).
const TRANCHES_SOUS_DIX: [&str; 5] = ["NN", "00", "01", "02", "03"];

/// Taille de l'entreprise d'après la catégorie et la tranche d'effectif de l'INSEE.
///
/// La catégorie (`PME`, `ETI`, `GE`) est la définition légale de la taille ; la tranche
/// d'effectif ne sert qu'à distinguer, au sein des PME, les très petites entreprises — et
/// la micro-entreprise d'un entrepreneur individuel (nature juridique `1000`).
#[must_use]
pub fn size_from_registry(
    size_category: Option<&str>,
    workforce_range: Option<&str>,
    legal_category: Option<&str>,
) -> Option<CompanySize> {
    let sous_dix = workforce_range.is_some_and(|range| TRANCHES_SOUS_DIX.contains(&range));
    let petite = |sous_dix: bool| {
        if !sous_dix {
            CompanySize::Pme
        } else if legal_category == Some("1000") {
            CompanySize::Micro
        } else {
            CompanySize::Tpe
        }
    };
    match size_category {
        Some("GE") => Some(CompanySize::Large),
        Some("ETI") => Some(CompanySize::Eti),
        Some("PME") => Some(petite(sous_dix)),
        _ => match workforce_range? {
            range if TRANCHES_SOUS_DIX.contains(&range) => Some(petite(true)),
            "11" | "12" | "21" | "22" | "31" => Some(CompanySize::Pme),
            "32" | "41" | "42" | "51" => Some(CompanySize::Eti),
            "52" | "53" => Some(CompanySize::Large),
            _ => None,
        },
    }
}

/// Valeur persistée d'une taille, telle que le formulaire l'attend.
#[must_use]
pub const fn size_code(size: CompanySize) -> &'static str {
    match size {
        CompanySize::Micro => "MICRO",
        CompanySize::Tpe => "TPE",
        CompanySize::Pme => "PME",
        CompanySize::Eti => "ETI",
        CompanySize::Large => "LARGE",
        CompanySize::Unknown => "UNKNOWN",
    }
}

// ---------------------------------------------------------------------------------------
// Normalisation
// ---------------------------------------------------------------------------------------

/// Mots laissés en minuscules au milieu d'un nom propre (« Rue de la Paix »).
const PARTICULES: [&str; 14] = [
    "de", "du", "des", "la", "le", "les", "et", "sur", "sous", "en", "aux", "au", "à", "lès",
];

/// Ramène un texte en capitales (« RUE DE LA PAIX ») à une casse lisible
/// (« Rue de la Paix »). Un texte déjà en casse mixte est conservé tel quel.
#[must_use]
pub fn readable_case(value: &str) -> String {
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if value.chars().any(char::is_lowercase) {
        return value;
    }
    value
        .split(' ')
        .enumerate()
        .map(|(index, word)| {
            let lower = word.to_lowercase();
            if index > 0 && PARTICULES.contains(&lower.as_str()) {
                return lower;
            }
            // « D'ARTAGNAN » → « d'Artagnan », « L'ÉGLISE » → « l'Église ».
            if let Some((article, reste)) = lower.split_once('\'') {
                if matches!(article, "d" | "l") && !reste.is_empty() {
                    let article = if index == 0 {
                        article.to_uppercase()
                    } else {
                        article.to_owned()
                    };
                    return format!("{article}'{}", compound(reste));
                }
            }
            compound(&lower)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Mot composé en minuscules : « saint-malo » → « Saint-Malo », « hay-les-roses » →
/// « Hay-les-Roses » (les particules intérieures restent en minuscules).
fn compound(word: &str) -> String {
    word.split('-')
        .enumerate()
        .map(|(index, part)| {
            if index > 0 && PARTICULES.contains(&part) {
                part.to_owned()
            } else {
                capitalize(part)
            }
        })
        .collect::<Vec<_>>()
        .join("-")
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + chars.as_str()
    })
}

/// Domaines d'annuaires, de réseaux sociaux et d'encyclopédies : jamais un site officiel.
const DOMAINES_NON_OFFICIELS: [&str; 30] = [
    "annuaire-entreprises.data.gouv.fr",
    "societe.com",
    "pappers.fr",
    "verif.com",
    "infogreffe.fr",
    "manageo.fr",
    "societeinfo.com",
    "corporama.com",
    "kompass.com",
    "pagesjaunes.fr",
    "linkedin.com",
    "facebook.com",
    "twitter.com",
    "x.com",
    "instagram.com",
    "youtube.com",
    "tiktok.com",
    "wikipedia.org",
    "wikidata.org",
    "indeed.com",
    "indeed.fr",
    "welcometothejungle.com",
    "glassdoor.fr",
    "glassdoor.com",
    "hellowork.com",
    "francetravail.fr",
    "google.com",
    "bing.com",
    "duckduckgo.com",
    "trustpilot.com",
];

/// Origine (`https://exemple.fr`) d'une URL publique, ou `None` si ce n'est pas un site
/// Web utilisable comme site officiel.
#[must_use]
pub fn website_origin(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url.trim()).ok()?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return None;
    }
    let host = parsed.host_str()?.to_ascii_lowercase();
    if !host.contains('.') {
        return None;
    }
    Some(format!("{}://{host}", parsed.scheme()))
}

/// L'URL désigne-t-elle un annuaire, un réseau social ou un moteur plutôt qu'un site
/// d'entreprise ?
#[must_use]
pub fn is_directory_site(url: &str) -> bool {
    let Some(host) = url::Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_ascii_lowercase))
    else {
        return true;
    };
    DOMAINES_NON_OFFICIELS
        .iter()
        .any(|domaine| host == *domaine || host.ends_with(&format!(".{domaine}")))
}

// ---------------------------------------------------------------------------------------
// Établissements candidats
// ---------------------------------------------------------------------------------------

/// Le nom de l'établissement correspond-il exactement à la recherche (casse, accents et
/// espaces ignorés) ?
fn same_name(candidate: &CompanyCandidate, query_key: &str) -> bool {
    std::iter::once(candidate.name.as_str())
        .chain(candidate.trade_name.as_deref())
        .any(|name| search_key(name) == query_key)
}

/// Trie les établissements — nom identique d'abord, puis sièges, puis ordre de l'annuaire —
/// restreint à la ville demandée quand elle est connue, et dit si le résultat est assez sûr
/// pour sauter l'étape de sélection.
///
/// « Assez sûr » veut dire : **un seul** établissement, au nom identique. Deux
/// établissements d'Orange, à Rennes et à Paris, ne le sont jamais — le choix revient à
/// l'utilisateur, l'application ne devine pas une ville.
#[must_use]
pub fn rank_candidates(
    query: &str,
    location: Option<&str>,
    mut candidates: Vec<CompanyCandidate>,
) -> CompanySearchResult {
    let mut seen = std::collections::HashSet::new();
    candidates.retain(|candidate| seen.insert(candidate.siret.clone()));
    if let Some(location) = location {
        let lieu = search_key(location);
        let locaux: Vec<CompanyCandidate> = candidates
            .iter()
            .filter(|candidate| {
                candidate
                    .city
                    .as_deref()
                    .is_some_and(|city| search_key(city).contains(&lieu))
                    || candidate.postal_code.as_deref() == Some(location)
            })
            .cloned()
            .collect();
        if !locaux.is_empty() {
            candidates = locaux;
        }
    }
    let query_key = search_key(query);
    candidates.sort_by_key(|candidate| (!same_name(candidate, &query_key), !candidate.head_office));
    let truncated = candidates.len() > MAX_CANDIDATES;
    candidates.truncate(MAX_CANDIDATES);
    let confident = candidates.len() == 1
        && candidates
            .first()
            .is_some_and(|candidate| same_name(candidate, &query_key));
    CompanySearchResult {
        candidates,
        confident,
        truncated,
    }
}

// ---------------------------------------------------------------------------------------
// Classification proposée par le modèle
// ---------------------------------------------------------------------------------------

/// Réponse du modèle : des **codes** pris dans les listes fournies, rien d'autre.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CompanyClassification {
    #[serde(default)]
    pub secteur: Option<String>,
    #[serde(default, rename = "type")]
    pub company_type: Option<String>,
    #[serde(default)]
    pub site: Option<String>,
}

impl ValidateAiOutput for CompanyClassification {
    fn validate_ai_output(&self) -> AppResult<()> {
        let trop_long = [&self.secteur, &self.company_type, &self.site]
            .iter()
            .any(|value| {
                value
                    .as_deref()
                    .is_some_and(|text| text.chars().count() > 64)
            });
        if trop_long {
            return Err(AppError::Provider(
                "Le modèle a renvoyé une classification illisible.".into(),
            ));
        }
        Ok(())
    }
}

/// Code court présenté au modèle pour le secteur d'indice `index` (`S01`, `S02`…).
#[must_use]
pub fn sector_code(index: usize) -> String {
    format!("S{:02}", index + 1)
}

/// Code court présenté au modèle pour l'URL d'indice `index` (`U1`, `U2`…).
#[must_use]
pub fn url_code(index: usize) -> String {
    format!("U{}", index + 1)
}

/// Classification retenue, une fois chaque code confronté aux listes fermées.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GroundedClassification {
    /// Identifiant du secteur dans le référentiel.
    pub sector_id: Option<String>,
    /// Code du type d'entreprise dans le référentiel.
    pub company_type: Option<String>,
    /// URL désignée, prise telle quelle dans la liste.
    pub website: Option<String>,
}

/// Recadre la réponse du modèle : un code absent des listes est écarté, jamais deviné.
#[must_use]
pub fn ground_classification(
    classification: &CompanyClassification,
    sector_ids: &[String],
    type_codes: &[String],
    urls: &[String],
) -> GroundedClassification {
    let normalise = |value: &Option<String>| {
        value
            .as_deref()
            .map(|text| text.trim().to_ascii_uppercase())
            .filter(|text| !text.is_empty() && text != "NULL")
    };
    let sector_id = normalise(&classification.secteur).and_then(|code| {
        (0..sector_ids.len())
            .find(|index| sector_code(*index) == code)
            .and_then(|index| sector_ids.get(index).cloned())
    });
    let company_type =
        normalise(&classification.company_type).filter(|code| type_codes.contains(code));
    let website = normalise(&classification.site).and_then(|code| {
        (0..urls.len())
            .find(|index| url_code(*index) == code)
            .and_then(|index| urls.get(index).cloned())
    });
    GroundedClassification {
        sector_id,
        company_type,
        website,
    }
}

#[cfg(test)]
#[path = "tests/company_research.rs"]
mod tests;
