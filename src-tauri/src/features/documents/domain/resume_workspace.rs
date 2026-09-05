//! Document de travail autonome d'un CV ciblé.

use crate::features::ai::domain::{AtsAnalysis, ContentRelevance, MatchScore, StructuredListing};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const RESUME_WORKSPACE_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeWorkspace {
    pub schema_version: u8,
    pub document: ResumeDocument,
    pub job_offer: StructuredListing,
    pub analysis: AtsAnalysis,
    pub score: MatchScore,
    pub initial_score: u8,
    pub proposals: Vec<ResumeProposal>,
    /// Bibliothèque optionnelle figée au moment de la génération. Un CV enregistré reste
    /// ainsi éditable même si le profil général change ensuite.
    #[serde(default)]
    pub profile_library: Vec<ResumeProfileItem>,
    /// Intentions explicites de l'utilisateur, conservées dans la session et à
    /// l'enregistrement du workspace.
    #[serde(default)]
    pub decisions: ResumeEditorialDecisions,
    /// Mesure issue du même moteur de composition que le PDF final.
    #[serde(default)]
    pub layout: ResumeLayoutMeasurement,
    /// Sous-ensemble prioritaire recalculé localement à partir des candidates IA.
    #[serde(default)]
    pub content_recommendations: Vec<ResumeContentRecommendation>,
    /// Erreur non bloquante de l'assistance IA. Les suggestions locales restent actives.
    #[serde(default)]
    pub recommendation_error: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeEditorialDecisions {
    pub explicitly_added: Vec<String>,
    pub explicitly_removed: Vec<String>,
    pub ignored: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub enum ResumeLayoutStatus {
    #[default]
    Spacious,
    Available,
    AlmostFull,
    Full,
    Overflow,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeLayoutMeasurement {
    pub status: ResumeLayoutStatus,
    /// Hauteur utilisée rapportée à la zone imprimable, bornée à 2000 (200 %).
    pub used_per_mille: u16,
    /// Place restante en points PDF ; négative en cas de dépassement.
    pub remaining_points: i32,
    pub page_count: u8,
    pub overflow: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeDocument {
    pub identity: ResumeIdentity,
    pub profile: String,
    pub experiences: Vec<ResumeExperienceBlock>,
    pub projects: Vec<ResumeProjectBlock>,
    pub skill_groups: Vec<ResumeSkillGroup>,
    pub education: Vec<ResumeEducationBlock>,
    pub certifications: Vec<ResumeCertificationBlock>,
    pub languages: Vec<ResumeLanguageBlock>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeIdentity {
    pub full_name: String,
    pub title: String,
    pub headline: Option<String>,
    pub city: Option<String>,
    pub phone: Option<String>,
    pub email: String,
    pub website: Option<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
    pub extra: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeExperienceBlock {
    pub id: String,
    pub title: String,
    pub company: String,
    pub location: Option<String>,
    pub period: String,
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeProjectBlock {
    pub id: String,
    pub name: String,
    pub meta: Option<String>,
    pub url: Option<String>,
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeSkillGroup {
    pub id: String,
    pub name: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeEducationBlock {
    pub id: String,
    pub degree: String,
    pub school: String,
    pub location: Option<String>,
    pub period: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeCertificationBlock {
    pub id: String,
    pub name: String,
    pub issuer: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeLanguageBlock {
    pub id: String,
    pub name: String,
    pub level: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub enum ResumeProfileItemContent {
    Skill { name: String },
    Project { value: ResumeProjectBlock },
    Certification { value: ResumeCertificationBlock },
    Language { value: ResumeLanguageBlock },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeProfileItem {
    pub id: String,
    pub label: String,
    pub detail: Option<String>,
    pub content: ResumeProfileItemContent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub enum ResumeContentRecommendationAction {
    Add {
        item_id: String,
    },
    Replace {
        add_item_id: String,
        remove_item_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeContentRecommendation {
    pub id: String,
    pub label: String,
    pub reason: String,
    pub relevance: ContentRelevance,
    pub action: ResumeContentRecommendationAction,
    pub layout_after: ResumeLayoutMeasurement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub enum ResumeProposalStatus {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub enum ResumeProposalKind {
    MissingSkill,
    TextReplacement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub enum ResumeProposalTarget {
    Profile,
    ExperienceDescription { experience_id: String },
    SkillGroup { group_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "documents.ts")]
pub struct ResumeProposal {
    pub id: String,
    pub kind: ResumeProposalKind,
    pub target: ResumeProposalTarget,
    pub label: String,
    pub original_text: Option<String>,
    pub proposed_text: String,
    pub gain: i16,
    pub status: ResumeProposalStatus,
    pub applicable: bool,
}
