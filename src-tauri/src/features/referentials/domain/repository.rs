//! Contrat d'accès aux référentiels métier.

use crate::core::errors::AppResult;
use crate::features::referentials::domain::catalog::Referentials;

/// Accès **en lecture seule** aux quatre catalogues.
///
/// Aucune écriture n'est exposée : les listes sont semées par `init_schema.sql`, ce qui
/// garantit des codes et des libellés identiques d'une installation à l'autre. Les faire
/// éditer par l'utilisateur casserait cette stabilité, dont dépendent les sauvegardes.
pub trait ReferentialRepository: Send + Sync {
    /// Charge les quatre référentiels dans leur ordre d'affichage.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si une requête échoue.
    fn load(&self) -> AppResult<Referentials>;
}
