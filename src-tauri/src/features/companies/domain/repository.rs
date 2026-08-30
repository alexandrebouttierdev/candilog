//! Contrat d'accès aux entreprises.

use crate::core::errors::AppResult;
use crate::core::pagination::Page;
use crate::features::companies::domain::company::{Company, NewCompany};
use crate::features::companies::domain::company_size::CompanySize;
use serde::{Deserialize, Serialize};

/// Critères du répertoire, appliqués par `SQLite` avant pagination.
///
/// Les trois dimensions restent séparées : le secteur qualifie l'activité, le type la
/// nature de l'organisation, la taille son effectif. Une entreprise peut être « ESN + PME »
/// comme « Association + TPE ».
#[derive(Debug, Clone, Default, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "companies.ts")]
pub struct CompanyFilter {
    /// Recherche libre sur le nom et la ville.
    #[serde(default)]
    pub search: String,
    /// Secteur d'activité retenu.
    pub sector_id: Option<uuid::Uuid>,
    /// Nature de l'organisation retenue.
    pub company_type_id: Option<String>,
    /// Taille retenue.
    pub company_size: Option<CompanySize>,
}

/// Accès au répertoire des entreprises.
///
/// Aucune méthode n'a d'implémentation par défaut : filtrer ou paginer en Rust après avoir
/// chargé toute la table donnerait un dépôt qui « fonctionne » tout en annulant la
/// pagination, et le défaut resterait invisible jusqu'à ce que le répertoire grossisse.
/// Chaque implémentation dit donc explicitement comment elle pagine.
pub trait CompanyRepository: Send + Sync {
    /// Liste toutes les entreprises, triées par nom.
    ///
    /// Réservé aux usages qui ont réellement besoin de l'ensemble — alimenter un sélecteur
    /// d'entreprise, par exemple. Les écrans de liste passent par [`list_page`](Self::list_page).
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<Company>>;

    /// Récupère une entreprise par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn get(&self, id: uuid::Uuid) -> AppResult<Company>;

    /// Renvoie une page filtrée par recherche libre, secteur, type et taille.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        filter: &CompanyFilter,
    ) -> AppResult<Page<Company>>;

    /// Crée une entreprise.
    ///
    /// # Errors
    /// `AppError::Validation` si le secteur ou le type référencé est inconnu.
    fn create(&self, input: &NewCompany) -> AppResult<Company>;

    /// Remplace les champs d'une entreprise.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn update(&self, id: uuid::Uuid, input: &NewCompany) -> AppResult<Company>;

    /// Supprime une entreprise.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu ; `AppError::Validation` si des
    /// candidatures y sont rattachées : la contrainte `ON DELETE RESTRICT` refuse alors la
    /// suppression.
    fn delete(&self, id: uuid::Uuid) -> AppResult<()>;
}
