//! Entité et champs éditables d'une candidature.

use crate::features::candidatures::domain::statut::{StatutCandidature, TypeContrat};
use serde::{Deserialize, Serialize};

/// Candidature telle que persistée, nom d'entreprise aplati depuis la jointure.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "candidatures.ts")]
pub struct Candidature {
    /// Identifiant de la candidature.
    pub id: uuid::Uuid,
    /// Intitulé du poste visé.
    pub poste: String,
    /// Identifiant de l'entreprise liée.
    pub entreprise_id: uuid::Uuid,
    /// Nom de l'entreprise liée, aplati depuis la jointure ; `None` si non résolu.
    pub entreprise_nom: Option<String>,
    /// Ville de l'entreprise liée, aplatie depuis la jointure.
    ///
    /// Affichée dans la colonne « Entreprise » de la vue Liste et sur les cartes du Kanban :
    /// sans elle, chaque ligne devrait relire le répertoire des entreprises.
    pub entreprise_ville: Option<String>,
    /// Identifiant du contact lié, s'il existe.
    pub contact_id: Option<uuid::Uuid>,
    /// Type de contrat visé.
    pub type_contrat: TypeContrat,
    /// Statut courant dans le pipeline.
    pub statut: StatutCandidature,
    /// Date d'envoi, au format `AAAA-MM-JJ`.
    ///
    /// Les lignes reprises de l'ancienne base peuvent porter un horodatage ISO 8601
    /// complet : le tri et l'affichage restent corrects, mais le format n'est pas homogène.
    pub date_envoi: String,
    /// Lien vers l'offre d'origine, s'il existe.
    pub lien_offre: Option<String>,
    /// Notes libres.
    pub notes: Option<String>,
    /// Date de création (ISO 8601).
    pub created_at: String,
    /// Date de dernière mise à jour (ISO 8601).
    pub updated_at: String,
}

/// Champs éditables d'une candidature, en création comme en modification.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "candidatures.ts")]
pub struct NouvelleCandidature {
    /// Intitulé du poste visé.
    pub poste: String,
    /// Identifiant de l'entreprise liée (requis).
    pub entreprise_id: uuid::Uuid,
    /// Type de contrat visé.
    pub type_contrat: TypeContrat,
    /// Statut initial ou cible.
    pub statut: StatutCandidature,
    /// Date d'envoi choisie par l'utilisateur, au format `AAAA-MM-JJ`.
    pub date_envoi: String,
    /// Lien vers l'offre, s'il existe.
    pub lien_offre: Option<String>,
    /// Notes libres.
    pub notes: Option<String>,
}
