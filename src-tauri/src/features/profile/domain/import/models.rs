use serde::{Deserialize, Serialize};

use super::super::{Certification, Education, Experience, Language, Project, Skill};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub enum ImportResolution {
    KeepExisting,
    Replace,
    AddAsNew,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct ImportScalarItem {
    pub id: String,
    pub label: String,
    pub proposed: String,
    pub existing: Option<String>,
    pub has_conflict: bool,
}

macro_rules! import_list_types {
    ($item:ident, $decision:ident, $ty:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
        #[serde(rename_all = "snake_case")]
        #[ts(export, export_to = "profile.ts")]
        pub struct $item {
            pub id: String,
            pub proposed: $ty,
            pub existing: Option<$ty>,
            #[ts(type = "number | null")]
            pub existing_index: Option<u32>,
            pub has_conflict: bool,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
        #[serde(rename_all = "snake_case")]
        #[ts(export, export_to = "profile.ts")]
        pub struct $decision {
            pub id: String,
            pub selected: bool,
            pub value: $ty,
            #[ts(type = "number | null")]
            pub existing_index: Option<u32>,
            pub resolution: ImportResolution,
        }
    };
}

import_list_types!(ImportExperienceItem, ImportExperienceDecision, Experience);
import_list_types!(ImportSkillItem, ImportSkillDecision, Skill);
import_list_types!(ImportEducationItem, ImportEducationDecision, Education);
import_list_types!(ImportLanguageItem, ImportLanguageDecision, Language);
import_list_types!(ImportProjectItem, ImportProjectDecision, Project);
import_list_types!(
    ImportCertificationItem,
    ImportCertificationDecision,
    Certification
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct ImportDetectedCounts {
    pub identity: u32,
    pub experiences: u32,
    pub skills: u32,
    pub education: u32,
    pub languages: u32,
    pub projects: u32,
    pub certifications: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct ImportProfilePreview {
    pub identity: Vec<ImportScalarItem>,
    pub experiences: Vec<ImportExperienceItem>,
    pub skills: Vec<ImportSkillItem>,
    pub education: Vec<ImportEducationItem>,
    pub languages: Vec<ImportLanguageItem>,
    pub projects: Vec<ImportProjectItem>,
    pub certifications: Vec<ImportCertificationItem>,
    pub counts: ImportDetectedCounts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct ImportScalarDecision {
    pub id: String,
    pub selected: bool,
    pub value: String,
    pub resolution: ImportResolution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct ImportProfileRequest {
    pub identity: Vec<ImportScalarDecision>,
    pub experiences: Vec<ImportExperienceDecision>,
    pub skills: Vec<ImportSkillDecision>,
    pub education: Vec<ImportEducationDecision>,
    pub languages: Vec<ImportLanguageDecision>,
    pub projects: Vec<ImportProjectDecision>,
    pub certifications: Vec<ImportCertificationDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "profile.ts")]
pub struct ImportProfileResult {
    pub added: u32,
    pub replaced: u32,
    pub skipped: u32,
}
