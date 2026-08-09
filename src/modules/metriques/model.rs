//! Types du domaine des métriques locales (télémétrie `LLM` + historique `ATS`).

use serde::{Deserialize, Serialize};

/// Opération `LLM` tracée par la télémétrie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationLlm {
    /// Extraction d'une offre (`parse_offer`).
    ParseOffer,
    /// Génération d'un CV reformulé (`generate_cv`).
    GenerateCv,
    /// Analyse `ATS` d'un CV (`analyze_ats`).
    AnalyzeAts,
    /// Structuration d'un CV importé (`parse_cv`).
    ParseCv,
    /// Analyse d'un compte rendu d'entretien (`analyser_entretien`).
    AnalyserEntretien,
    /// Génération d'une lettre de motivation (`generer_lettre_motivation`).
    CoverLetter,
}

impl OperationLlm {
    /// Libellé stable stocké en base (identique au tag `serde`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ParseOffer => "parse_offer",
            Self::GenerateCv => "generate_cv",
            Self::AnalyzeAts => "analyze_ats",
            Self::ParseCv => "parse_cv",
            Self::AnalyserEntretien => "analyser_entretien",
            Self::CoverLetter => "cover_letter",
        }
    }

    /// Reconstruit l'opération depuis son libellé stocké (`None` si inconnu).
    #[must_use]
    pub fn depuis_str(s: &str) -> Option<Self> {
        match s {
            "parse_offer" => Some(Self::ParseOffer),
            "generate_cv" => Some(Self::GenerateCv),
            "analyze_ats" => Some(Self::AnalyzeAts),
            "parse_cv" => Some(Self::ParseCv),
            "analyser_entretien" => Some(Self::AnalyserEntretien),
            "cover_letter" => Some(Self::CoverLetter),
            _ => None,
        }
    }
}

/// Origine d'un score `ATS` enregistré.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrigineScore {
    /// Score d'un CV généré (`generate_cv`).
    Genere,
    /// Score d'un CV importé (`analyze_imported_cv`).
    Importe,
}

impl OrigineScore {
    /// Libellé stable stocké en base (identique au tag `serde`).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Genere => "genere",
            Self::Importe => "importe",
        }
    }

    /// Reconstruit l'origine depuis son libellé stocké (`None` si inconnu).
    #[must_use]
    pub fn depuis_str(s: &str) -> Option<Self> {
        match s {
            "genere" => Some(Self::Genere),
            "importe" => Some(Self::Importe),
            _ => None,
        }
    }
}

/// Enregistrement d'un appel `LLM` (une opération logique, retries inclus).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppelLlm {
    /// Opération tracée.
    pub operation: OperationLlm,
    /// Fournisseur (`ollama`/`claude`/`openai`/`gemini`/`mistral`/`custom`).
    pub provider: String,
    /// Nom du modèle utilisé.
    pub modele: String,
    /// Temps mur de l'opération en millisecondes.
    pub latence_ms: u64,
    /// Vrai si l'opération a réussi.
    pub succes: bool,
    /// Horodatage `ISO 8601` (`RFC 3339`, `UTC`), injecté par l'appelant.
    pub cree_le: String,
}

/// Enregistrement d'un score `ATS` produit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreAts {
    /// Score `ATS` (0–100).
    pub score: u8,
    /// Origine du score (généré ou importé).
    pub origine: OrigineScore,
    /// Horodatage `ISO 8601` (`RFC 3339`, `UTC`), injecté par l'appelant.
    pub cree_le: String,
}

/// Page bornée de résultats, accompagnée des informations de navigation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Page<T> {
    /// Éléments de la page courante.
    pub items: Vec<T>,
    /// Nombre total d'éléments, toutes pages confondues.
    pub total: u64,
    /// Numéro de page courant, à partir de 1.
    pub page: u64,
    /// Nombre maximal d'éléments par page.
    pub page_size: u64,
    /// Nombre total de pages (au moins 1, même si l'historique est vide).
    pub total_pages: u64,
}

impl<T> Page<T> {
    /// Construit les métadonnées d'une page à partir de son total.
    #[must_use]
    pub fn new(items: Vec<T>, total: u64, page: u64, page_size: u64) -> Self {
        let page_size = page_size.max(1);
        let total_pages = total.div_ceil(page_size).max(1);
        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

/// Agrégats globaux des scores `ATS`, calculés directement par `SQLite`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResumeScoresAts {
    /// Nombre total de scores.
    pub nombre: u64,
    /// Moyenne globale arrondie.
    pub moyenne: u8,
    /// Scores compris entre 0 et 49.
    pub faibles: u64,
    /// Scores compris entre 50 et 69.
    pub partiels: u64,
    /// Scores compris entre 70 et 84.
    pub bons: u64,
    /// Scores compris entre 85 et 100.
    pub excellents: u64,
    /// Nombre de scores issus de CV générés.
    pub generes_nombre: u64,
    /// Moyenne arrondie des CV générés.
    pub generes_moyenne: u8,
    /// Nombre de scores issus de CV importés.
    pub importes_nombre: u64,
    /// Moyenne arrondie des CV importés.
    pub importes_moyenne: u8,
}

/// Page de scores avec agrégats globaux indépendants de la page courante.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageScoresAts {
    /// Page d'historique bornée.
    #[serde(flatten)]
    pub pagination: Page<ScoreAts>,
    /// Statistiques calculées sur l'historique complet.
    pub resume: ResumeScoresAts,
}

#[cfg(test)]
#[path = "tests/model/mod.rs"]
mod tests;
