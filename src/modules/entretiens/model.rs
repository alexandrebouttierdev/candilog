//! Types du domaine des entretiens.

use crate::shared::types::AnalyseEntretien;
use serde::{Deserialize, Serialize};

/// Type d'entretien (valeurs de l'enum Postgres `type_entretien`, casse exacte).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TypeEntretien {
    /// Entretien en présentiel (défaut).
    #[default]
    #[serde(rename = "Présentiel")]
    Presentiel,
    /// Entretien en visioconférence.
    #[serde(rename = "Visio")]
    Visio,
    /// Entretien téléphonique.
    #[serde(rename = "Téléphonique")]
    Telephonique,
    /// Entretien technique.
    #[serde(rename = "Technique")]
    Technique,
    /// Entretien avec les ressources humaines.
    #[serde(rename = "RH")]
    Rh,
    /// Autre type d'entretien.
    #[serde(rename = "Autre")]
    Autre,
}

impl std::fmt::Display for TypeEntretien {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Presentiel => "Présentiel",
            Self::Visio => "Visio",
            Self::Telephonique => "Téléphonique",
            Self::Technique => "Technique",
            Self::Rh => "RH",
            Self::Autre => "Autre",
        })
    }
}

/// Entretien rattaché à une candidature, tel que persisté.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entretien {
    /// Identifiant de l'entretien.
    pub id: uuid::Uuid,
    /// Identifiant de la candidature (FK `candidatures`, requis).
    pub candidature_id: uuid::Uuid,
    /// Identifiant du contact lié (FK `contacts`), s'il existe.
    pub contact_id: Option<uuid::Uuid>,
    /// Date et heure de l'entretien (ISO 8601).
    pub date_entretien: String,
    /// Type d'entretien.
    #[serde(rename = "type")]
    pub type_entretien: TypeEntretien,
    /// Lieu (présentiel) ou lien (visio), si renseigné.
    pub lieu: Option<String>,
    /// Notes de préparation, si renseignées.
    pub notes: Option<String>,
    /// Compte rendu après l'entretien, s'il est renseigné.
    pub compte_rendu: Option<String>,
    /// Analyse `IA` du compte rendu, si elle a déjà été produite.
    ///
    /// `#[serde(default)]` : stockée en `NULL` tant qu'aucune analyse n'a été enregistrée, et
    /// absente des fixtures de test antérieures à son introduction.
    #[serde(default)]
    pub analyse_ia: Option<AnalyseEntretien>,
    /// Date de création (ISO 8601).
    pub created_at: String,
    /// Date de dernière mise à jour (ISO 8601).
    pub updated_at: String,
}

/// Champs de création/édition d'un entretien (`candidature_id` et `date_entretien` requis).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NouvelEntretien {
    /// Identifiant de la candidature (requis).
    pub candidature_id: uuid::Uuid,
    /// Identifiant du contact lié (optionnel).
    pub contact_id: Option<uuid::Uuid>,
    /// Date et heure de l'entretien (ISO 8601).
    pub date_entretien: String,
    /// Type d'entretien.
    #[serde(rename = "type")]
    pub type_entretien: TypeEntretien,
    /// Lieu ou lien.
    pub lieu: Option<String>,
    /// Notes de préparation.
    pub notes: Option<String>,
    /// Compte rendu.
    pub compte_rendu: Option<String>,
}

/// Champs d'édition d'un entretien (remplacement complet, identique à la création).
pub type MajEntretien = NouvelEntretien;

#[cfg(test)]
#[path = "tests/model/mod.rs"]
mod tests;
