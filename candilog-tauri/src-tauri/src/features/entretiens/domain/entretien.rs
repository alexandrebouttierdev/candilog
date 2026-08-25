//! Entité, type et champs éditables d'un entretien.

use serde::{Deserialize, Serialize};

/// Format de l'entretien.
///
/// Les valeurs sérialisées reprennent la casse exacte contrainte en base par le `CHECK` de
/// la migration 005, accents compris : les renommer romprait la lecture des lignes existantes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "entretiens.ts")]
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
    /// Autre format.
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

/// Analyse `IA` du compte rendu d'un entretien.
///
/// Persistée en `TEXT` `JSON` sur l'entretien. Définie ici plutôt que dans la feature `ia` :
/// c'est un champ de l'entretien, et la faire vivre ailleurs obligerait `entretiens` à
/// dépendre de l'IA pour lire ses propres lignes.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "entretiens.ts")]
pub struct AnalyseEntretien {
    /// Résumé synthétique de l'entretien.
    pub resume: String,
    /// Points forts relevés dans le compte rendu.
    pub points_forts: Vec<String>,
    /// Points faibles relevés dans le compte rendu.
    pub points_faibles: Vec<String>,
    /// Suggestions pour les prochains entretiens.
    pub suggestions: Vec<String>,
}

/// Entretien rattaché à une candidature, tel que persisté.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "entretiens.ts")]
pub struct Entretien {
    /// Identifiant de l'entretien.
    pub id: uuid::Uuid,
    /// Identifiant de la candidature concernée.
    pub candidature_id: uuid::Uuid,
    /// Intitulé du poste, aplati depuis la jointure — ce que le calendrier affiche.
    pub candidature_poste: Option<String>,
    /// Nom de l'entreprise, aplati depuis la jointure.
    pub entreprise_nom: Option<String>,
    /// Identifiant du contact lié, s'il existe.
    pub contact_id: Option<uuid::Uuid>,
    /// Nom complet de l'interlocuteur, aplati depuis la jointure.
    pub contact_nom: Option<String>,
    /// Date et heure de l'entretien (ISO 8601).
    pub date_entretien: String,
    /// Format de l'entretien.
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub type_entretien: TypeEntretien,
    /// Lieu (présentiel) ou lien (visio).
    pub lieu: Option<String>,
    /// Notes de préparation.
    pub notes: Option<String>,
    /// Compte rendu rédigé après l'entretien.
    pub compte_rendu: Option<String>,
    /// Analyse `IA` du compte rendu, si elle a été produite.
    #[serde(default)]
    pub analyse_ia: Option<AnalyseEntretien>,
    /// Date de création (ISO 8601).
    pub created_at: String,
    /// Date de dernière mise à jour (ISO 8601).
    pub updated_at: String,
}

/// Champs éditables d'un entretien, en création comme en modification.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "entretiens.ts")]
pub struct NouvelEntretien {
    /// Identifiant de la candidature concernée (requis).
    pub candidature_id: uuid::Uuid,
    /// Identifiant du contact lié.
    pub contact_id: Option<uuid::Uuid>,
    /// Date et heure de l'entretien (ISO 8601).
    pub date_entretien: String,
    /// Format de l'entretien.
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub type_entretien: TypeEntretien,
    /// Lieu ou lien.
    pub lieu: Option<String>,
    /// Notes de préparation.
    pub notes: Option<String>,
    /// Compte rendu.
    pub compte_rendu: Option<String>,
}
