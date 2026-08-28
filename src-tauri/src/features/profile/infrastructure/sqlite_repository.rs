//! Dépôt SQLite compatible avec le JSON historique de l'application Iced.

use crate::core::database::helpers::{connection, now_iso, translate_error};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::profile::domain::{
    Certification, Skill, Experience, Education, Identity, Language, Profile, ProfileRepository,
    Project,
};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

/// Dépôt de la ligne singleton `profil`.
pub struct SqliteProfileRepository {
    pool: SqlitePool,
}

impl SqliteProfileRepository {
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl ProfileRepository for SqliteProfileRepository {
    fn get(&self) -> AppResult<(Profile, Option<String>)> {
        let conn = connection(&self.pool)?;
        let row: Option<(String, String)> = conn
            .query_row(
                "SELECT data, updated_at FROM profile WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|error| translate_error(error, "profil"))?;
        match row {
            None => Ok((Profile::default(), None)),
            Some((json, updated_at)) => {
                let stored: StoredProfile = serde_json::from_str(&json)
                    .map_err(|error| AppError::Serialization(error.to_string()))?;
                Ok((stored.into(), Some(updated_at)))
            }
        }
    }

    fn save(&self, profile: &Profile) -> AppResult<(Profile, String)> {
        let conn = connection(&self.pool)?;
        let updated_at = now_iso();
        // Le format de stockage reste celui de l'application Iced (`personal.first_name`,
        // `skills`, `education`…) afin qu'une base existante reste lisible dans les deux apps.
        let json = serde_json::to_string(&StoredProfile::from(profile))
            .map_err(|error| AppError::Serialization(error.to_string()))?;
        conn.execute(
            "INSERT INTO profile (id, data, updated_at) VALUES (1, ?1, ?2)
             ON CONFLICT(id) DO UPDATE SET data = excluded.data, updated_at = excluded.updated_at",
            rusqlite::params![json, updated_at],
        )
        .map_err(|error| translate_error(error, "profil"))?;
        Ok((profile.clone(), updated_at))
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct StoredProfile {
    personal: StoredIdentity,
    experiences: Vec<ExperienceStored>,
    skills: Vec<SkillStored>,
    education: Vec<EducationStored>,
    languages: Vec<LanguageStored>,
    projects: Vec<ProjectStored>,
    certifications: Vec<CertificationStored>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct StoredIdentity {
    first_name: String,
    last_name: String,
    email: String,
    phone: Option<String>,
    city: Option<String>,
    headline: Option<String>,
    summary: Option<String>,
    linkedin: Option<String>,
    github: Option<String>,
    website: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct ExperienceStored {
    title: String,
    company: String,
    location: Option<String>,
    start_date: String,
    end_date: Option<String>,
    current: bool,
    description: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct SkillStored {
    name: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct EducationStored {
    degree: String,
    school: String,
    location: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct LanguageStored {
    name: String,
    level: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct ProjectStored {
    name: String,
    description: Option<String>,
    url: Option<String>,
    technologies: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct CertificationStored {
    name: String,
    issuer: Option<String>,
    date: Option<String>,
    url: Option<String>,
}

impl From<StoredProfile> for Profile {
    fn from(value: StoredProfile) -> Self {
        Self {
            identity: value.personal.into(),
            experiences: value.experiences.into_iter().map(Into::into).collect(),
            skills: value.skills.into_iter().map(Into::into).collect(),
            education: value.education.into_iter().map(Into::into).collect(),
            languages: value.languages.into_iter().map(Into::into).collect(),
            projects: value.projects.into_iter().map(Into::into).collect(),
            certifications: value.certifications.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<&Profile> for StoredProfile {
    fn from(value: &Profile) -> Self {
        Self {
            personal: (&value.identity).into(),
            experiences: value.experiences.iter().map(Into::into).collect(),
            skills: value.skills.iter().map(Into::into).collect(),
            education: value.education.iter().map(Into::into).collect(),
            languages: value.languages.iter().map(Into::into).collect(),
            projects: value.projects.iter().map(Into::into).collect(),
            certifications: value.certifications.iter().map(Into::into).collect(),
        }
    }
}

impl From<StoredIdentity> for Identity {
    fn from(value: StoredIdentity) -> Self {
        Self {
            first_name: value.first_name,
            name: value.last_name,
            email: value.email,
            phone: value.phone,
            city: value.city,
            title: value.headline,
            resume: value.summary,
            linkedin: value.linkedin,
            github: value.github,
            website: value.website,
        }
    }
}

impl From<&Identity> for StoredIdentity {
    fn from(value: &Identity) -> Self {
        Self {
            first_name: value.first_name.clone(),
            last_name: value.name.clone(),
            email: value.email.clone(),
            phone: value.phone.clone(),
            city: value.city.clone(),
            headline: value.title.clone(),
            summary: value.resume.clone(),
            linkedin: value.linkedin.clone(),
            github: value.github.clone(),
            website: value.website.clone(),
        }
    }
}

impl From<ExperienceStored> for Experience {
    fn from(value: ExperienceStored) -> Self {
        Self {
            title: value.title,
            company: value.company,
            location: value.location,
            start_date: value.start_date,
            end_date: value.end_date,
            current: value.current,
            description: value.description,
        }
    }
}

impl From<&Experience> for ExperienceStored {
    fn from(value: &Experience) -> Self {
        Self {
            title: value.title.clone(),
            company: value.company.clone(),
            location: value.location.clone(),
            start_date: value.start_date.clone(),
            end_date: value.end_date.clone(),
            current: value.current,
            description: value.description.clone(),
        }
    }
}

macro_rules! conversion_simple {
    ($stored:ty, $domaine:ty, $($stock:ident => $domain:ident),+ $(,)?) => {
        impl From<$stored> for $domaine {
            fn from(value: $stored) -> Self {
                Self { $($domain: value.$stock),+ }
            }
        }
        impl From<&$domaine> for $stored {
            fn from(value: &$domaine) -> Self {
                Self { $($stock: value.$domain.clone()),+ }
            }
        }
    };
}

conversion_simple!(SkillStored, Skill, name => name);
conversion_simple!(
    EducationStored,
    Education,
    degree => degree,
    school => school,
    location => location,
    start_date => start_date,
    end_date => end_date,
    description => description,
);
conversion_simple!(LanguageStored, Language, name => name, level => level);
conversion_simple!(
    ProjectStored,
    Project,
    name => name,
    description => description,
    url => url,
    technologies => technologies,
);
conversion_simple!(
    CertificationStored,
    Certification,
    name => name,
    issuer => issuer,
    date => date,
    url => url,
);

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
