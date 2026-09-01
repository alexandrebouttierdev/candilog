//! Types échangés avec React pour les workflows IA.

use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

fn string_lenient<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::String(v) => v,
        serde_json::Value::Number(v) => v.to_string(),
        serde_json::Value::Bool(v) => v.to_string(),
        serde_json::Value::Array(v) => v
            .into_iter()
            .map(|i| i.as_str().unwrap_or_default().to_owned())
            .collect::<Vec<_>>()
            .join(", "),
        serde_json::Value::Object(v) => v
            .into_values()
            .filter_map(|i| i.as_str().map(str::to_owned))
            .collect::<Vec<_>>()
            .join(" — "),
        serde_json::Value::Null => String::new(),
    })
}

fn list_lenient<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Array(items) => items
            .into_iter()
            .filter_map(|v| match v {
                serde_json::Value::String(s) => Some(s),
                serde_json::Value::Number(n) => Some(n.to_string()),
                _ => None,
            })
            .collect(),
        serde_json::Value::String(s) => vec![s],
        _ => Vec::new(),
    })
}

/// Retombe sur `Profile` pour toute valeur absente ou inconnue (ex. `"resume"` des anciennes
/// réponses IA) : une section imprévue ne doit jamais faire échouer la désérialisation d'une
/// analyse historique, seulement dégrader la recommandation vers la section sans exigence
/// d'`item_index`.
fn section_lenient<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<AtsRecommendationSection, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::String(s) if s.eq_ignore_ascii_case("experience") => {
            AtsRecommendationSection::Experience
        }
        _ => AtsRecommendationSection::Profile,
    })
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct StructuredListing {
    #[serde(default, alias = "titre", deserialize_with = "string_lenient")]
    pub title: String,
    #[serde(default, alias = "competences", deserialize_with = "list_lenient")]
    pub skills: Vec<String>,
    #[serde(
        default,
        alias = "savoirEtre",
        alias = "savoir_etre",
        deserialize_with = "list_lenient"
    )]
    pub soft_skills: Vec<String>,
    #[serde(default)]
    pub experience: Option<String>,
    #[serde(
        default,
        alias = "motsCles",
        alias = "mots_cles",
        deserialize_with = "list_lenient"
    )]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct MatchScore {
    pub total: u8,
    pub skills: Option<u8>,
    pub experience: Option<u8>,
    pub ats: Option<u8>,
    pub present: Vec<String>,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ListingAnalysis {
    pub job_offer: StructuredListing,
    pub score: MatchScore,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct GeneratedExperience {
    #[serde(default, alias = "intitule", deserialize_with = "string_lenient")]
    pub title: String,
    #[serde(default, alias = "entreprise", deserialize_with = "string_lenient")]
    pub company: String,
    #[serde(default, deserialize_with = "string_lenient")]
    pub description: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct GeneratedEducation {
    #[serde(default, alias = "diplome", deserialize_with = "string_lenient")]
    pub degree: String,
    #[serde(default, alias = "etablissement", deserialize_with = "string_lenient")]
    pub school: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct GeneratedResume {
    #[serde(default, deserialize_with = "string_lenient")]
    pub resume: String,
    #[serde(default)]
    pub experiences: Vec<GeneratedExperience>,
    #[serde(default, deserialize_with = "list_lenient")]
    pub skills: Vec<String>,
    #[serde(default, alias = "formations")]
    pub education: Vec<GeneratedEducation>,
}

/// Section ciblée par une recommandation ATS. Le score et les suggestions libres du LLM ne
/// sont plus exposés (`AtsAnalysis`) : seule une recommandation dont la cible est identifiable
/// — et donc simulable puis applicable — reste présentée comme une action possible.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub enum AtsRecommendationSection {
    #[default]
    Profile,
    Experience,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct AtsRecommendation {
    #[serde(default, deserialize_with = "section_lenient")]
    pub section: AtsRecommendationSection,
    // Indice (0-based) de l'expérience du CV ciblée. Requis pour `Experience`, absent pour
    // `Profile` : validé par `validate_ai_output`.
    #[serde(default)]
    pub item_index: Option<usize>,
    #[serde(
        default,
        alias = "texteOriginal",
        alias = "texte_original",
        deserialize_with = "string_lenient"
    )]
    pub original_text: String,
    #[serde(
        default,
        alias = "textePropose",
        alias = "texte_propose",
        deserialize_with = "string_lenient"
    )]
    pub proposed_text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct AtsAnalysis {
    #[serde(default, alias = "summary", deserialize_with = "string_lenient")]
    pub recap: String,
    #[serde(default, alias = "recommandations")]
    pub recommendations: Vec<AtsRecommendation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ResumeGeneration {
    pub resume: GeneratedResume,
    pub analysis: AtsAnalysis,
    pub job_offer: StructuredListing,
    pub profile_score: MatchScore,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ResumeGenerationRequest {
    pub generation_id: String,
    pub job_offer: String,
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct CoverLetterRequest {
    pub generation_id: String,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub tone: Option<String>,
    pub length: Option<String>,
    pub context: Option<String>,
    pub previous_cover_letter: Option<String>,
    pub instruction: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ImportedResumeAnalysis {
    pub resume: GeneratedResume,
    pub job_offer: StructuredListing,
    pub score: MatchScore,
    pub analysis: AtsAnalysis,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ResumeAnalysisRequest {
    pub generation_id: String,
    pub job_offer: String,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ProfileImportRequest {
    pub generation_id: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct AiProgress {
    pub generation_id: String,
    pub step: String,
    pub chunk: Option<String>,
}

/// Progression d'analyse de CV : étape connue et ligne de journal, sans pourcentage.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ProfileImportProgress {
    pub generation_id: String,
    pub at: String,
    pub message: String,
    pub step: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Une analyse enregistrée avant la fermeture du contrat portait `score`, `suggestions`
    /// et un `impact` par recommandation, et une section libre (`"resume"`) sans `item_index`.
    /// Elle doit rester lisible pour l'historique (bibliothèque de CV, duplication) sans
    /// jamais réexposer ce score LLM.
    #[test]
    fn une_analyse_historique_reste_lisible_sans_le_score_llm() {
        let historique = r#"{
            "score": 70,
            "recap": "CV solide",
            "suggestions": ["Ajouter Docker"],
            "recommendations": [
                {"section": "resume", "texte_original": "Ancien profil", "texte_propose": "Nouveau profil", "impact": 80}
            ]
        }"#;

        let analysis: AtsAnalysis = serde_json::from_str(historique).unwrap();

        assert_eq!(analysis.recap, "CV solide");
        assert_eq!(analysis.recommendations.len(), 1);
        assert_eq!(
            analysis.recommendations[0].section,
            AtsRecommendationSection::Profile
        );
        assert_eq!(analysis.recommendations[0].item_index, None);
        assert_eq!(analysis.recommendations[0].original_text, "Ancien profil");
    }

    #[test]
    fn une_section_experience_connue_est_reconnue() {
        let recommendation: AtsRecommendation =
            serde_json::from_str(r#"{"section": "experience", "item_index": 2}"#).unwrap();

        assert_eq!(recommendation.section, AtsRecommendationSection::Experience);
        assert_eq!(recommendation.item_index, Some(2));
    }
}
