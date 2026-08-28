//! Dépôt SQLite compatible avec le JSON historique de l'application Iced.

use crate::core::database::helpers::{connexion, maintenant_iso, traduire_erreur};
use crate::core::database::SqlitePool;
use crate::core::errors::{AppError, AppResult};
use crate::features::profil::domain::{
    Certification, Competence, Experience, Formation, Identite, Langue, Profil, ProfilRepository,
    Projet,
};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

/// Dépôt de la ligne singleton `profil`.
pub struct SqliteProfilRepository {
    pool: SqlitePool,
}

impl SqliteProfilRepository {
    #[must_use]
    pub const fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl ProfilRepository for SqliteProfilRepository {
    fn obtenir(&self) -> AppResult<(Profil, Option<String>)> {
        let conn = connexion(&self.pool)?;
        let ligne: Option<(String, String)> = conn
            .query_row(
                "SELECT data, updated_at FROM profil WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|error| traduire_erreur(error, "profil"))?;
        match ligne {
            None => Ok((Profil::default(), None)),
            Some((json, updated_at)) => {
                let stocke: ProfilStocke = serde_json::from_str(&json)
                    .map_err(|error| AppError::Serialization(error.to_string()))?;
                Ok((stocke.into(), Some(updated_at)))
            }
        }
    }

    fn enregistrer(&self, profil: &Profil) -> AppResult<(Profil, String)> {
        let conn = connexion(&self.pool)?;
        let updated_at = maintenant_iso();
        // Le format de stockage reste celui de l'application Iced (`personal.first_name`,
        // `skills`, `education`…) afin qu'une base existante reste lisible dans les deux apps.
        let json = serde_json::to_string(&ProfilStocke::from(profil))
            .map_err(|error| AppError::Serialization(error.to_string()))?;
        conn.execute(
            "INSERT INTO profil (id, data, updated_at) VALUES (1, ?1, ?2)
             ON CONFLICT(id) DO UPDATE SET data = excluded.data, updated_at = excluded.updated_at",
            rusqlite::params![json, updated_at],
        )
        .map_err(|error| traduire_erreur(error, "profil"))?;
        Ok((profil.clone(), updated_at))
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct ProfilStocke {
    personal: IdentiteStockee,
    experiences: Vec<ExperienceStockee>,
    skills: Vec<CompetenceStockee>,
    education: Vec<FormationStockee>,
    languages: Vec<LangueStockee>,
    projects: Vec<ProjetStocke>,
    certifications: Vec<CertificationStockee>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct IdentiteStockee {
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
struct ExperienceStockee {
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
struct CompetenceStockee {
    name: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct FormationStockee {
    degree: String,
    school: String,
    location: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct LangueStockee {
    name: String,
    level: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct ProjetStocke {
    name: String,
    description: Option<String>,
    url: Option<String>,
    technologies: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct CertificationStockee {
    name: String,
    issuer: Option<String>,
    date: Option<String>,
    url: Option<String>,
}

impl From<ProfilStocke> for Profil {
    fn from(value: ProfilStocke) -> Self {
        Self {
            identite: value.personal.into(),
            experiences: value.experiences.into_iter().map(Into::into).collect(),
            competences: value.skills.into_iter().map(Into::into).collect(),
            formations: value.education.into_iter().map(Into::into).collect(),
            langues: value.languages.into_iter().map(Into::into).collect(),
            projets: value.projects.into_iter().map(Into::into).collect(),
            certifications: value.certifications.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<&Profil> for ProfilStocke {
    fn from(value: &Profil) -> Self {
        Self {
            personal: (&value.identite).into(),
            experiences: value.experiences.iter().map(Into::into).collect(),
            skills: value.competences.iter().map(Into::into).collect(),
            education: value.formations.iter().map(Into::into).collect(),
            languages: value.langues.iter().map(Into::into).collect(),
            projects: value.projets.iter().map(Into::into).collect(),
            certifications: value.certifications.iter().map(Into::into).collect(),
        }
    }
}

impl From<IdentiteStockee> for Identite {
    fn from(value: IdentiteStockee) -> Self {
        Self {
            prenom: value.first_name,
            nom: value.last_name,
            email: value.email,
            telephone: value.phone,
            ville: value.city,
            titre: value.headline,
            resume: value.summary,
            linkedin: value.linkedin,
            github: value.github,
            site_web: value.website,
        }
    }
}

impl From<&Identite> for IdentiteStockee {
    fn from(value: &Identite) -> Self {
        Self {
            first_name: value.prenom.clone(),
            last_name: value.nom.clone(),
            email: value.email.clone(),
            phone: value.telephone.clone(),
            city: value.ville.clone(),
            headline: value.titre.clone(),
            summary: value.resume.clone(),
            linkedin: value.linkedin.clone(),
            github: value.github.clone(),
            website: value.site_web.clone(),
        }
    }
}

impl From<ExperienceStockee> for Experience {
    fn from(value: ExperienceStockee) -> Self {
        Self {
            intitule: value.title,
            entreprise: value.company,
            lieu: value.location,
            date_debut: value.start_date,
            date_fin: value.end_date,
            poste_actuel: value.current,
            description: value.description,
        }
    }
}

impl From<&Experience> for ExperienceStockee {
    fn from(value: &Experience) -> Self {
        Self {
            title: value.intitule.clone(),
            company: value.entreprise.clone(),
            location: value.lieu.clone(),
            start_date: value.date_debut.clone(),
            end_date: value.date_fin.clone(),
            current: value.poste_actuel,
            description: value.description.clone(),
        }
    }
}

macro_rules! conversion_simple {
    ($stocke:ty, $domaine:ty, $($stock:ident => $domain:ident),+ $(,)?) => {
        impl From<$stocke> for $domaine {
            fn from(value: $stocke) -> Self {
                Self { $($domain: value.$stock),+ }
            }
        }
        impl From<&$domaine> for $stocke {
            fn from(value: &$domaine) -> Self {
                Self { $($stock: value.$domain.clone()),+ }
            }
        }
    };
}

conversion_simple!(CompetenceStockee, Competence, name => nom);
conversion_simple!(
    FormationStockee,
    Formation,
    degree => diplome,
    school => etablissement,
    location => lieu,
    start_date => date_debut,
    end_date => date_fin,
    description => description,
);
conversion_simple!(LangueStockee, Langue, name => nom, level => niveau);
conversion_simple!(
    ProjetStocke,
    Projet,
    name => nom,
    description => description,
    url => url,
    technologies => technologies,
);
conversion_simple!(
    CertificationStockee,
    Certification,
    name => nom,
    issuer => organisme,
    date => date,
    url => url,
);

#[cfg(test)]
#[path = "tests/sqlite_repository/mod.rs"]
mod tests;
