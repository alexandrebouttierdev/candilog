//! Types de l'analyse d'offre et du scoring `ATS`.

use serde::{Deserialize, Deserializer, Serialize};

/// Convertit une valeur `JSON` scalaire en chaîne.
///
/// Les `LLM` renvoient parfois un nombre ou un booléen là où une chaîne est
/// attendue (ex. `"experience": 3`). On coerce les scalaires ; les objets,
/// tableaux et `null` sont ignorés (`None`).
fn value_to_string(value: serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Convertit aussi les structures produites spontanément par certains LLM en
/// texte lisible. Un objet à une propriété comme `{ "poste": "Développeur" }`
/// ne doit pas faire échouer toute l'analyse du CV.
fn value_to_text(value: serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s,
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Array(items) => items
            .into_iter()
            .map(value_to_text)
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join(", "),
        serde_json::Value::Object(fields) => fields
            .into_values()
            .map(value_to_text)
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join(" — "),
        serde_json::Value::Null => String::new(),
    }
}

/// Désérialise un texte de façon tolérante pour les réponses JSON des LLM.
fn de_lenient_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(value_to_text(serde_json::Value::deserialize(deserializer)?))
}

/// Désérialise un champ optionnel en tolérant les scalaires non-chaîne du `LLM`.
///
/// # Errors
///
/// Never fails: any unexpected `JSON` shape falls back to `None`.
fn de_lenient_opt_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(value_to_string(value))
}

/// Désérialise un tableau de chaînes en coerçant les éléments scalaires du `LLM`.
///
/// # Errors
///
/// Never fails: non-array inputs and non-scalar elements are dropped.
fn de_lenient_string_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Array(items) => {
            Ok(items.into_iter().filter_map(value_to_string).collect())
        }
        other => Ok(value_to_string(other).into_iter().collect()),
    }
}

/// Désérialise un score renvoyé sous forme de nombre ou de chaîne et le borne
/// à l'intervalle ATS attendu.
fn de_lenient_u8<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let score = match value {
        serde_json::Value::Number(number) => number.as_u64().unwrap_or_default(),
        serde_json::Value::String(text) => text
            .trim()
            .trim_end_matches('%')
            .parse::<u64>()
            .unwrap_or_default(),
        _ => 0,
    };
    Ok(score.min(100) as u8)
}

/// Offre d'emploi analysée (extraite par le `LLM`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ParsedOffer {
    /// Intitulé du poste.
    pub title: String,
    /// Compétences techniques requises.
    #[serde(default, deserialize_with = "de_lenient_string_vec")]
    pub skills: Vec<String>,
    /// Compétences comportementales.
    #[serde(default, deserialize_with = "de_lenient_string_vec")]
    pub soft_skills: Vec<String>,
    /// Expérience requise (texte libre, ex. « 3 ans »).
    #[serde(default, deserialize_with = "de_lenient_opt_string")]
    pub experience: Option<String>,
    /// Mots-clés de l'offre.
    #[serde(default, deserialize_with = "de_lenient_string_vec")]
    pub keywords: Vec<String>,
}

#[cfg(test)]
#[path = "tests/cv_model/mod.rs"]
mod tests;

/// Score de correspondance profil × offre (0–100).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MatchScore {
    /// Score total pondéré.
    pub total: u8,
    /// Sous-score compétences (40 %).
    pub skills: u8,
    /// Sous-score expérience (40 %).
    pub experience: u8,
    /// Sous-score mots-clés `ATS` (20 %).
    pub ats: u8,
    /// Compétences de l'offre présentes dans le profil.
    pub matched: Vec<String>,
    /// Compétences de l'offre absentes du profil.
    pub missing: Vec<String>,
}

/// Résultat complet de l'analyse d'une offre (retour de commande).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfferAnalysis {
    /// Offre analysée.
    pub parsed: ParsedOffer,
    /// Score calculé.
    pub score: MatchScore,
}

/// Expérience reformulée dans le CV généré.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GeneratedExperience {
    /// Intitulé du poste.
    #[serde(default, deserialize_with = "de_lenient_string")]
    pub title: String,
    /// Entreprise.
    #[serde(default, deserialize_with = "de_lenient_string")]
    pub company: String,
    /// Description reformulée.
    #[serde(default, deserialize_with = "de_lenient_string")]
    pub description: String,
}

/// Formation dans le CV généré.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GeneratedEducation {
    /// Diplôme.
    #[serde(default, deserialize_with = "de_lenient_string")]
    pub degree: String,
    /// Établissement.
    #[serde(default, deserialize_with = "de_lenient_string")]
    pub school: String,
}

/// Désérialise les expériences sous forme structurée ou sous forme de simples
/// phrases, format fréquemment produit par les modèles compacts.
fn de_lenient_experience_vec<'de, D>(deserializer: D) -> Result<Vec<GeneratedExperience>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let items = match value {
        serde_json::Value::Array(items) => items,
        serde_json::Value::Null => return Ok(Vec::new()),
        other => vec![other],
    };

    Ok(items
        .into_iter()
        .filter_map(|item| {
            if item.is_object() {
                let fallback = value_to_text(item.clone());
                let parsed = serde_json::from_value::<GeneratedExperience>(item).ok()?;
                if parsed.title.is_empty()
                    && parsed.company.is_empty()
                    && parsed.description.is_empty()
                {
                    (!fallback.is_empty()).then(|| GeneratedExperience {
                        description: fallback,
                        ..Default::default()
                    })
                } else {
                    Some(parsed)
                }
            } else {
                let description = value_to_text(item);
                (!description.is_empty()).then(|| GeneratedExperience {
                    description,
                    ..Default::default()
                })
            }
        })
        .collect())
}

/// Désérialise également une formation résumée en une simple chaîne.
fn de_lenient_education_vec<'de, D>(deserializer: D) -> Result<Vec<GeneratedEducation>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let items = match value {
        serde_json::Value::Array(items) => items,
        serde_json::Value::Null => return Ok(Vec::new()),
        other => vec![other],
    };

    Ok(items
        .into_iter()
        .filter_map(|item| {
            if item.is_object() {
                let fallback = value_to_text(item.clone());
                let parsed = serde_json::from_value::<GeneratedEducation>(item).ok()?;
                if parsed.degree.is_empty() && parsed.school.is_empty() {
                    (!fallback.is_empty()).then(|| GeneratedEducation {
                        degree: fallback,
                        ..Default::default()
                    })
                } else {
                    Some(parsed)
                }
            } else {
                let degree = value_to_text(item);
                (!degree.is_empty()).then(|| GeneratedEducation {
                    degree,
                    ..Default::default()
                })
            }
        })
        .collect())
}

/// CV généré et reformulé pour une offre.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GeneratedCv {
    /// Résumé adapté à l'offre.
    #[serde(default, deserialize_with = "de_lenient_string")]
    pub summary: String,
    /// Expériences reformulées.
    #[serde(default, deserialize_with = "de_lenient_experience_vec")]
    pub experiences: Vec<GeneratedExperience>,
    /// Compétences mises en avant.
    #[serde(default, deserialize_with = "de_lenient_string_vec")]
    pub skills: Vec<String>,
    /// Formations.
    #[serde(default, deserialize_with = "de_lenient_education_vec")]
    pub education: Vec<GeneratedEducation>,
}

/// Recommandation `ATS` structurée, applicable à un champ du CV (accepter/refuser).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RecommandationAts {
    /// Cible parsable du champ concerné : `resume`, `competences` ou `experience_<i>`.
    #[serde(default, deserialize_with = "de_lenient_string")]
    pub section: String,
    /// Texte actuel (avant amélioration).
    #[serde(default, deserialize_with = "de_lenient_string")]
    pub texte_original: String,
    /// Texte proposé par l'`IA` (après amélioration).
    #[serde(default, deserialize_with = "de_lenient_string")]
    pub texte_propose: String,
    /// Gain estimé de points `ATS` si la recommandation est appliquée.
    #[serde(default, deserialize_with = "de_lenient_u8")]
    pub impact: u8,
}

/// Analyse `ATS` du CV généré.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AtsAnalysis {
    /// Score `ATS` (0–100).
    #[serde(
        default,
        alias = "score_ats",
        alias = "ats_score",
        alias = "total",
        deserialize_with = "de_lenient_u8"
    )]
    pub score: u8,
    /// Synthèse lisible de l'adéquation du CV avec l'offre.
    #[serde(
        default,
        alias = "resume",
        alias = "summary",
        deserialize_with = "de_lenient_string"
    )]
    pub recap: String,
    /// Suggestions actionnables (max 5).
    #[serde(default, deserialize_with = "de_lenient_string_vec")]
    pub suggestions: Vec<String>,
    /// Recommandations structurées applicables (accepter/refuser). Optionnel pour
    /// rester compatible avec les anciennes réponses `LLM` sans ce champ.
    #[serde(default)]
    pub recommandations: Vec<RecommandationAts>,
}

/// Résultat complet de la génération de CV (retour de commande).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CvGeneration {
    /// CV généré.
    pub cv: GeneratedCv,
    /// Analyse `ATS` du CV généré.
    pub analysis: AtsAnalysis,
}

/// Lettre de motivation générée (miroir du contrat de l'Edge Function `generate-cover-letter`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CoverLetter {
    /// Corps de la lettre de motivation (texte brut, en français).
    #[serde(default, deserialize_with = "de_lenient_string")]
    pub lettre: String,
}

/// Message de l'historique du chat d'itération d'une lettre de motivation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMsg {
    /// Émetteur du message : `"user"` ou `"assistant"`.
    pub role: String,
    /// Contenu textuel du message.
    pub content: String,
}

/// Demande unifiée de génération de lettre (entrée de commande), pour les deux modes.
///
/// `source` distingue le mode **offre** (`"job_offer"`) du mode **demande personnalisée**
/// (`"custom_request"`). En itération, `previous_letter` + `instruction` (+ `chat_history`)
/// pilotent la ré-écriture. Le profil est repris côté backend ; `profile` ne sert qu'aux
/// corrections transitoires (non persistées).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LetterGenerationRequest {
    /// Origine : `"job_offer"` ou `"custom_request"`.
    #[serde(default)]
    pub source: String,
    /// Identifiant client de la génération (annulation, corrélation des événements de streaming).
    #[serde(default)]
    pub generation_id: String,
    /// Demande libre de l'utilisateur (mode personnalisé) — instruction complémentaire.
    #[serde(default)]
    pub user_instruction: Option<String>,
    /// Nom de l'entreprise ciblée.
    #[serde(default)]
    pub company_name: Option<String>,
    /// Intitulé du poste ou de la formation visée.
    #[serde(default)]
    pub job_title: Option<String>,
    /// Type de contrat (CDI, alternance, professionnalisation, stage…).
    #[serde(default)]
    pub contract_type: Option<String>,
    /// Type de candidature : `"job_offer"` (réponse à une offre) ou `"spontaneous"` (spontanée).
    #[serde(default)]
    pub application_type: Option<String>,
    /// Description/compétences de l'offre (collée par l'utilisateur en mode offre).
    #[serde(default)]
    pub job_description: Option<String>,
    /// Secteur d'activité (utilisé quand ni offre ni entreprise précise ne sont fournies).
    #[serde(default)]
    pub sector: Option<String>,
    /// Ton souhaité : `"formal"`, `"casual"` ou `"creative"` (défaut `formal` si absent).
    #[serde(default)]
    pub tone: Option<String>,
    /// Longueur souhaitée : `"short"`, `"medium"` ou `"long"`.
    #[serde(default)]
    pub length: Option<String>,
    /// Profil corrigé avant génération (transitoire, non persisté). Absent/vide → profil stocké.
    #[serde(default)]
    pub profile: Option<crate::shared::profile::Profile>,
    /// Lettre précédente à retravailler (itération) ; `None` = première génération.
    #[serde(default)]
    pub previous_letter: Option<String>,
    /// Instruction d'itération en langage naturel (ex. « conclusion plus percutante »).
    #[serde(default)]
    pub instruction: Option<String>,
    /// Historique de la conversation d'itération (contexte des échanges).
    #[serde(default)]
    pub chat_history: Vec<ChatMsg>,
}

/// Résultat de l'analyse d'une demande libre (extraction structurée avant génération).
///
/// Produit soit par le `LLM` (`analyser_demande_lettre`), soit par un repli heuristique. Chaque
/// champ absent laisse le pipeline compléter avec le profil et n'affiche une question que si
/// l'information reste vraiment manquante.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandeLettre {
    /// Entreprise détectée.
    #[serde(default, deserialize_with = "de_lenient_opt_string")]
    pub company_name: Option<String>,
    /// Poste ou formation détecté.
    #[serde(default, deserialize_with = "de_lenient_opt_string")]
    pub job_title: Option<String>,
    /// Type de contrat détecté.
    #[serde(default, deserialize_with = "de_lenient_opt_string")]
    pub contract_type: Option<String>,
    /// Type de candidature détecté (`"job_offer"` / `"spontaneous"`).
    #[serde(default, deserialize_with = "de_lenient_opt_string")]
    pub application_type: Option<String>,
    /// Ton détecté.
    #[serde(default, deserialize_with = "de_lenient_opt_string")]
    pub tone: Option<String>,
    /// Longueur détectée.
    #[serde(default, deserialize_with = "de_lenient_opt_string")]
    pub length: Option<String>,
    /// Points clés à mettre en avant, extraits de la demande.
    #[serde(default, deserialize_with = "de_lenient_string_vec")]
    pub key_points: Vec<String>,
}
