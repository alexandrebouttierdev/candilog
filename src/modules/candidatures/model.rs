//! Types du domaine des candidatures.

use serde::{Deserialize, Serialize};

/// Statut d'une candidature (valeurs de l'enum Postgres `statut_candidature`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatutCandidature {
    /// En attente (défaut).
    EnAttente,
    /// Relancée après envoi.
    Relancee,
    /// En phase d'entretien.
    Entretien,
    /// Refusée.
    Refus,
}

impl std::fmt::Display for StatutCandidature {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::EnAttente => "En attente",
            Self::Relancee => "Relancée",
            Self::Entretien => "Entretien",
            Self::Refus => "Refusée",
        })
    }
}

/// Type de contrat (valeurs de l'enum Postgres `type_contrat`, casse exacte).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeContrat {
    /// Contrat à durée indéterminée.
    #[serde(rename = "CDI")]
    Cdi,
    /// Contrat à durée déterminée.
    #[serde(rename = "CDD")]
    Cdd,
    /// Mission freelance.
    #[serde(rename = "Freelance")]
    Freelance,
    /// Stage.
    #[serde(rename = "Stage")]
    Stage,
    /// Alternance.
    #[serde(rename = "Alternance")]
    Alternance,
    /// Intérim.
    #[serde(rename = "Interim")]
    Interim,
    /// Autre type de contrat.
    #[serde(rename = "Autre")]
    Autre,
}

impl std::fmt::Display for TypeContrat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Cdi => "CDI",
            Self::Cdd => "CDD",
            Self::Freelance => "Freelance",
            Self::Stage => "Stage",
            Self::Alternance => "Alternance",
            Self::Interim => "Intérim",
            Self::Autre => "Autre",
        })
    }
}

/// Candidature du domaine (nom d'entreprise aplati depuis la jointure, lecture seule).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidature {
    /// Identifiant de la candidature.
    pub id: uuid::Uuid,
    /// Intitulé du poste visé.
    pub poste: String,
    /// Identifiant de l'entreprise liée (FK `entreprises`).
    pub entreprise_id: uuid::Uuid,
    /// Nom de l'entreprise liée (aplati depuis la jointure ; `None` si non résolu).
    pub entreprise_nom: Option<String>,
    /// Identifiant du contact lié (FK `contacts`), s'il existe (lecture seule).
    pub contact_id: Option<uuid::Uuid>,
    /// Type de contrat visé.
    pub type_contrat: TypeContrat,
    /// Statut courant dans le pipeline.
    pub statut: StatutCandidature,
    /// Date d'envoi, au format `AAAA-MM-JJ` écrit par le formulaire.
    ///
    /// Les lignes reprises de l'ancienne base peuvent porter un horodatage ISO 8601 complet :
    /// le tri et l'affichage restent corrects, mais le format n'est pas homogène.
    pub date_envoi: String,
    /// Lien vers l'offre d'origine, s'il existe.
    pub lien_offre: Option<String>,
    /// Notes libres de l'utilisateur.
    pub notes: Option<String>,
    /// Date de création (ISO 8601).
    pub created_at: String,
    /// Date de dernière mise à jour (ISO 8601).
    pub updated_at: String,
}

/// Champs éditables d'une candidature (création et mise à jour).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NouvelleCandidature {
    /// Intitulé du poste visé.
    pub poste: String,
    /// Identifiant de l'entreprise liée (FK `entreprises`).
    pub entreprise_id: uuid::Uuid,
    /// Type de contrat visé.
    pub type_contrat: TypeContrat,
    /// Statut initial ou cible.
    pub statut: StatutCandidature,
    /// Date d'envoi choisie par l'utilisateur (`ISO 8601`).
    pub date_envoi: String,
    /// Lien vers l'offre, s'il existe.
    pub lien_offre: Option<String>,
    /// Notes libres.
    pub notes: Option<String>,
}

#[cfg(test)]
#[path = "tests/model/mod.rs"]
mod tests;
