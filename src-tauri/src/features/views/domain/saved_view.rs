//! Vue enregistrée : un filtre de Candidatures et son nom.

use crate::features::applications::domain::ApplicationFilter;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Longueur maximale du nom d'une vue, en caractères : il tient dans la navigation.
pub const MAX_VIEW_NAME: usize = 60;

/// Vue enregistrée, telle que la navigation la liste.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "views.ts")]
pub struct SavedView {
    pub id: Uuid,
    pub name: String,
    /// Filtre rejoué à l'ouverture, recherche libre comprise.
    pub filter: ApplicationFilter,
    /// Rang dans la navigation.
    #[ts(type = "number")]
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Création ou modification d'une vue.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "views.ts")]
pub struct NewSavedView {
    pub name: String,
    pub filter: ApplicationFilter,
}
