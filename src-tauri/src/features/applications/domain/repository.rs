//! Contrat d'accès aux candidatures.

use crate::core::errors::AppResult;
use crate::core::pagination::Page;
use crate::features::applications::domain::application::{Application, NewApplication};
use crate::features::applications::domain::application_type::ApplicationType;
use crate::features::applications::domain::schedule::WeeklyWorkSchedule;
use crate::features::applications::domain::status::ApplicationStatus;
use crate::features::companies::domain::CompanySize;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Colonne de tri de la vue Liste.
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
    /// Nom de l'entreprise.
    Company,
    /// Statut dans le pipeline.
    Status,
    /// Date d'envoi, ordre par défaut.
    #[default]
    Date,
}

/// Critères appliqués par `SQLite` avant pagination.
///
/// Tous les critères sont évalués en base : ramener les candidatures en mémoire pour les
/// filtrer en React reviendrait à charger tout le pipeline à chaque frappe.
///
/// La ville et le type d'entreprise portent sur la **valeur effective** — surcharge de la
/// candidature si elle existe, valeur de l'entreprise sinon —, faute de quoi une
/// candidature qui hérite de la ville de son entreprise échapperait au filtre « Rennes ».
#[derive(Debug, Clone, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "applications.ts")]
pub struct ApplicationFilter {
    /// Recherche libre sur le poste et le nom de l'entreprise.
    pub search: String,
    /// Statuts retenus ; vide = tous.
    #[serde(default)]
    pub status: Vec<ApplicationStatus>,
    /// Natures de candidature retenues ; vide = toutes.
    #[serde(default)]
    pub application_type: Vec<ApplicationType>,
    /// Codes de contrat retenus ; vide = tous.
    #[serde(default)]
    pub contract_type_code: Vec<String>,
    /// Domaines professionnels retenus ; vide = tous.
    #[serde(default)]
    pub professional_domain_id: Vec<String>,
    /// Types d'entreprise effectifs retenus ; vide = tous.
    #[serde(default)]
    pub company_type_id: Vec<String>,
    /// Tailles d'entreprise retenues ; vide = toutes. Portée par l'entreprise liée.
    #[serde(default)]
    pub company_size: Vec<CompanySize>,
    /// Secteurs d'activité de l'entreprise retenus ; vide = tous.
    #[serde(default)]
    pub sector_id: Vec<Uuid>,
    /// Régimes horaires retenus ; vide = tous.
    #[serde(default)]
    pub weekly_work_schedule: Vec<WeeklyWorkSchedule>,
    /// Borne basse du volume horaire hebdomadaire.
    pub min_weekly_hours: Option<f64>,
    /// Borne haute du volume horaire hebdomadaire.
    pub max_weekly_hours: Option<f64>,
    /// Entreprise liée.
    pub company_id: Option<Uuid>,
    /// Ville effective, en recherche partielle.
    pub city: String,
    /// Intitulé de poste, en recherche partielle.
    pub job_title: String,
    /// Borne basse de la date d'envoi (`AAAA-MM-JJ`).
    pub start_date: Option<String>,
    /// Borne haute de la date d'envoi (`AAAA-MM-JJ`).
    pub end_date: Option<String>,
    /// Colonne de tri.
    pub sort: ApplicationSort,
    /// Tri descendant.
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
    /// Nombre de candidatures en attente.
    #[ts(type = "number")]
    pub pending: u64,
    /// Nombre de candidatures relancées.
    #[ts(type = "number")]
    pub followed_up: u64,
    /// Nombre de candidatures en entretien.
    #[ts(type = "number")]
    pub interview: u64,
    /// Nombre de candidatures refusées.
    #[ts(type = "number")]
    pub rejected: u64,
}

/// Accès aux candidatures.
pub trait ApplicationRepository: Send + Sync {
    /// Liste toutes les candidatures, les plus récentes d'abord.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<Application>>;

    /// Récupère une candidature par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn get(&self, id: Uuid) -> AppResult<Application>;

    /// Renvoie une page après filtrage et tri.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        filter: &ApplicationFilter,
    ) -> AppResult<Page<Application>>;

    /// Compte les candidatures par statut, sans charger les lignes.
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
