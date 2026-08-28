//! Types échangés avec React pour les workflows IA.

use crate::features::profile::domain::Profile;
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

fn score_lenient<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u8, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    let score = match value {
        serde_json::Value::Number(n) => n.as_u64().unwrap_or_default(),
        serde_json::Value::String(s) => s.trim_end_matches('%').parse().unwrap_or_default(),
        _ => 0,
    };
    Ok(score.min(100) as u8)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct StructuredListing {
    #[serde(default, deserialize_with = "string_lenient")]
    pub title: String,
    #[serde(default, deserialize_with = "list_lenient")]
    pub skills: Vec<String>,
    #[serde(default, deserialize_with = "list_lenient")]
    pub soft_skills: Vec<String>,
    #[serde(default)]
    pub experience: Option<String>,
    #[serde(default, deserialize_with = "list_lenient")]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct MatchScore {
    pub total: u8,
    pub skills: u8,
    pub experience: u8,
    pub ats: u8,
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
    #[serde(default, deserialize_with = "string_lenient")]
    pub title: String,
    #[serde(default, deserialize_with = "string_lenient")]
    pub company: String,
    #[serde(default, deserialize_with = "string_lenient")]
    pub description: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct GeneratedEducation {
    #[serde(default, deserialize_with = "string_lenient")]
    pub degree: String,
    #[serde(default, deserialize_with = "string_lenient")]
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
    #[serde(default)]
    pub education: Vec<GeneratedEducation>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct AtsRecommendation {
    pub section: String,
    pub original_text: String,
    pub proposed_text: String,
    #[serde(default, deserialize_with = "score_lenient")]
    pub impact: u8,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct AtsAnalysis {
    #[serde(default, alias = "score_ats", deserialize_with = "score_lenient")]
    pub score: u8,
    #[serde(default, alias = "summary", deserialize_with = "string_lenient")]
    pub recap: String,
    #[serde(default, deserialize_with = "list_lenient")]
    pub suggestions: Vec<String>,
    #[serde(default)]
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
    pub path: String,
    pub job_offer: String,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ProfileImportRequest {
    pub generation_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct ExtractedProfile {
    pub profile: Profile,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct AiProgress {
    pub generation_id: String,
    pub step: String,
    pub progress: u8,
    pub chunk: Option<String>,
}
