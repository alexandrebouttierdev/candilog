//! Historique d'une relation (`screens/05-companies.png`, inspecteur « Historique »).
//!
//! L'historique ne montre que des faits enregistrés : candidatures envoyées, changements de
//! statut, entretiens, relances faites, notes saisies et ajout de la fiche. Rien n'est
//! reconstitué : une date inventée se lirait comme vraie.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Longueur maximale d'une note, en caractères.
pub const MAX_NOTE_CHARS: usize = 2000;
/// Entrées d'historique renvoyées au plus, les plus récentes d'abord.
pub const MAX_HISTORY: usize = 100;

/// Nature de la fiche dont on lit l'historique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "relations.ts")]
pub enum RelationKind {
    Company,
    Contact,
}

/// Entreprise ou contact visé.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "relations.ts")]
pub struct RelationRef {
    pub kind: RelationKind,
    pub id: Uuid,
}

/// Nature d'une entrée d'historique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "relations.ts")]
pub enum HistoryKind {
    Note,
    Interview,
    FollowUpDone,
    StatusChanged,
    ApplicationSent,
    Added,
}

/// Entrée d'historique, mise en mots par l'interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "relations.ts")]
pub struct HistoryEntry {
    pub kind: HistoryKind,
    /// Date `AAAA-MM-JJ` (envoi, note) ou horodatage RFC 3339 (statut, entretien, relance,
    /// ajout de la fiche).
    pub at: String,
    /// Candidature concernée ; absente pour une note ou l'ajout de la fiche.
    pub application_id: Option<Uuid>,
    pub job_title: Option<String>,
    /// Statut atteint, format d'entretien, canal de relance ou texte de la note.
    pub detail: Option<String>,
    /// Identifiant de la note, pour la supprimer.
    pub note_id: Option<Uuid>,
}

/// Note saisie depuis l'inspecteur.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "relations.ts")]
pub struct NewRelationNote {
    pub relation: RelationRef,
    pub body: String,
    /// Date `AAAA-MM-JJ` du fait noté ; peut précéder la saisie.
    pub noted_on: String,
}

/// Note enregistrée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "relations.ts")]
pub struct RelationNote {
    pub id: Uuid,
    pub relation: RelationRef,
    pub body: String,
    pub noted_on: String,
    pub created_at: String,
}
