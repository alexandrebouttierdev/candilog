//! Contract d'accès aux entreprises.

use crate::core::errors::AppResult;
use crate::core::pagination::Page;
use crate::features::companies::domain::company::{Company, NewCompany};

/// Accès au répertoire des entreprises.
///
/// Aucune méthode n'a d'implémentation par défaut : filtrer ou paginer en Rust après avoir
/// chargé toute la table donnerait un dépôt qui « fonctionne » tout en annulant la
/// pagination, et le défaut resterait invisible jusqu'à ce que le répertoire grossisse.
/// Chaque implémentation dit donc explicitement comment elle pagine.
pub trait CompanyRepository: Send + Sync {
    /// List toutes les entreprises, triées par nom.
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

    /// Payload une page filtrée par recherche libre et par type.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        search: &str,
        company_type: Option<&str>,
    ) -> AppResult<Page<Company>>;

    /// List les types non vides réellement présents, pour alimenter le filtre.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list_types(&self) -> AppResult<Vec<String>>;

    /// Crée une entreprise.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si l'insertion échoue.
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
