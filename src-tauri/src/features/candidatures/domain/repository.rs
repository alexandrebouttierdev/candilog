//! Contrat d'accès aux candidatures.

use crate::core::errors::AppResult;
use crate::core::pagination::Page;
use crate::features::candidatures::domain::candidature::{Candidature, NouvelleCandidature};
use crate::features::candidatures::domain::statut::{StatutCandidature, TypeContrat};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Colonne de tri de la vue Liste.
///
/// Enum et non chaîne libre : la valeur est interpolée dans le `ORDER BY`, où une chaîne
/// venue de l'IPC ouvrirait une injection. Le jeu fermé rend celle-ci impossible sans avoir
/// à échapper quoi que ce soit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "candidatures.ts")]
pub enum TriCandidature {
    /// Intitulé du poste.
    Poste,
    /// Nom de l'entreprise.
    Entreprise,
    /// Statut dans le pipeline.
    Statut,
    /// Date d'envoi, ordre par défaut.
    #[default]
    Date,
}

/// Critères appliqués par `SQLite` avant pagination.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "candidatures.ts")]
pub struct FiltreCandidatures {
    /// Recherche libre sur le poste et le nom de l'entreprise.
    pub search: String,
    /// Statut exact.
    pub statut: Option<StatutCandidature>,
    /// Type de contrat exact.
    pub contrat: Option<TypeContrat>,
    /// Entreprise liée.
    pub entreprise_id: Option<Uuid>,
    /// Ville de l'entreprise liée, en recherche partielle.
    pub ville: String,
    /// Intitulé de poste, en recherche partielle.
    pub poste: String,
    /// Borne basse de la date d'envoi (`AAAA-MM-JJ`).
    pub date_debut: Option<String>,
    /// Borne haute de la date d'envoi (`AAAA-MM-JJ`).
    pub date_fin: Option<String>,
    /// Colonne de tri.
    pub tri: TriCandidature,
    /// Tri descendant.
    pub descendant: bool,
}

/// Répartition du pipeline par statut, calculée par `SQLite`.
#[derive(Debug, Clone, Default, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "candidatures.ts")]
pub struct RepartitionPipeline {
    /// Nombre de candidatures en attente.
    #[ts(type = "number")]
    pub en_attente: u64,
    /// Nombre de candidatures relancées.
    #[ts(type = "number")]
    pub relancee: u64,
    /// Nombre de candidatures en entretien.
    #[ts(type = "number")]
    pub entretien: u64,
    /// Nombre de candidatures refusées.
    #[ts(type = "number")]
    pub refus: u64,
}

/// Accès aux candidatures.
pub trait CandidatureRepository: Send + Sync {
    /// Liste toutes les candidatures, les plus récentes d'abord.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<Candidature>>;

    /// Récupère une candidature par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn get(&self, id: Uuid) -> AppResult<Candidature>;

    /// Charge une page après filtrage et tri.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        filtre: &FiltreCandidatures,
    ) -> AppResult<Page<Candidature>>;

    /// Compte les candidatures par statut, sans charger les lignes.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn repartition(&self, filtre: &FiltreCandidatures) -> AppResult<RepartitionPipeline>;

    /// Crée une candidature et ouvre son historique de statut.
    ///
    /// # Errors
    /// `AppError::Validation` si l'entreprise liée est introuvable.
    fn create(&self, input: &NouvelleCandidature) -> AppResult<Candidature>;

    /// Remplace les champs d'une candidature, en historisant un éventuel changement de statut.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn update(&self, id: Uuid, input: &NouvelleCandidature) -> AppResult<Candidature>;

    /// Change le seul statut, en l'historisant — c'est le geste du Kanban.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn update_statut(&self, id: Uuid, statut: StatutCandidature) -> AppResult<Candidature>;

    /// Supprime une candidature ; ses relances, entretiens et historique suivent en cascade.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la suppression échoue.
    fn delete(&self, id: Uuid) -> AppResult<()>;
}
