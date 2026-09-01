//! Document de travail autonome d'un CV ciblé.

use crate::features::ai::domain::{AtsAnalysis, MatchScore, StructuredListing};
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
