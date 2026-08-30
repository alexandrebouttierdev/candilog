//! Contract d'accès aux candidatures.

use crate::core::errors::AppResult;
use crate::core::pagination::Page;
use crate::features::applications::domain::application::{Application, NewApplication};
use crate::features::applications::domain::status::{ApplicationStatus, ContractType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Column de tri de la vue List.
///
/// Enum et non chaîne libre : la valeur est interpolée dans le `ORDER BY`, où une chaîne
/// venue de l'IPC ouvrirait une injection. Le jeu fermé rend celle-ci impossible sans avoir
/// à échapper quoi que ce soit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "applications.ts")]
pub enum ApplicationSort {
    /// Intitulé du poste.
    JobTitle,
    /// Name de l'entreprise.
    Company,
    /// Status dans le pipeline.
    Status,
    /// Date d'envoi, ordre par défaut.
    #[default]
    Date,
}

/// Critères appliqués par `SQLite` avant pagination.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "applications.ts")]
pub struct ApplicationFilter {
    /// Recherche libre sur le poste et le nom de l'entreprise.
    pub search: String,
    /// Statuts retenus ; vide = tous.
    #[serde(default)]
    pub status: Vec<ApplicationStatus>,
    /// Types de contrat retenus ; vide = tous.
    #[serde(default)]
    pub contract: Vec<ContractType>,
    /// Company liée.
    pub company_id: Option<Uuid>,
    /// City de l'entreprise liée, en recherche partielle.
    pub city: String,
    /// Intitulé de poste, en recherche partielle.
    pub job_title: String,
    /// Borne basse de la date d'envoi (`AAAA-MM-JJ`).
    pub start_date: Option<String>,
    /// Borne haute de la date d'envoi (`AAAA-MM-JJ`).
    pub end_date: Option<String>,
    /// Column de tri.
    pub sort: ApplicationSort,
    /// Sort descendant.
    pub descending: bool,
    /// Identifiants retenus pour un export ou une action groupée ; vide = tout le filtre.
    #[serde(default)]
    pub ids: Vec<Uuid>,
}

/// Répartition du pipeline par statut, calculée par `SQLite`.
#[derive(Debug, Clone, Default, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "applications.ts")]
pub struct PipelineBreakdown {
    /// Count de candidatures en attente.
    #[ts(type = "number")]
    pub pending: u64,
    /// Count de candidatures relancées.
    #[ts(type = "number")]
    pub followed_up: u64,
    /// Count de candidatures en entretien.
    #[ts(type = "number")]
    pub interview: u64,
    /// Count de candidatures refusées.
    #[ts(type = "number")]
    pub rejected: u64,
}

/// Accès aux candidatures.
pub trait ApplicationRepository: Send + Sync {
    /// List toutes les candidatures, les plus récentes d'abord.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<Application>>;

    /// Récupère une candidature par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn get(&self, id: Uuid) -> AppResult<Application>;

    /// Payload une page après filtrage et tri.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        filter: &ApplicationFilter,
    ) -> AppResult<Page<Application>>;

    /// Report les candidatures par statut, sans charger les lignes.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn breakdown(&self, filter: &ApplicationFilter) -> AppResult<PipelineBreakdown>;

    /// Crée une candidature et ouvre son historique de statut.
    ///
    /// # Errors
    /// `AppError::Validation` si l'entreprise liée est introuvable.
    fn create(&self, input: &NewApplication) -> AppResult<Application>;

    /// Remplace les champs d'une candidature, en historisant un éventuel changement de statut.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn update(&self, id: Uuid, input: &NewApplication) -> AppResult<Application>;

    /// Change le seul statut, en l'historisant — c'est le geste du Kanban.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn update_status(&self, id: Uuid, status: ApplicationStatus) -> AppResult<Application>;

    /// Supprime une candidature ; ses relances, entretiens et historique suivent en cascade.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu ; `AppError::Database` si la
    /// suppression échoue.
    fn delete(&self, id: Uuid) -> AppResult<()>;
}
