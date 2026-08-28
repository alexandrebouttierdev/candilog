//! Contract d'accès au référentiel des secteurs.

use crate::core::errors::AppResult;
use crate::features::sectors::domain::sector::ActivitySector;

/// Accès en lecture au référentiel des secteurs d'activité.
///
/// Le référentiel est en lecture seule pour l'application : il est alimenté au démarrage
/// depuis la liste canonique, jamais par l'utilisateur. Un secteur ne s'ajoute qu'en
/// modifiant [`SECTORS_CANONIQUES`](crate::features::secteurs::infrastructure::SECTORS_CANONIQUES),
/// ce qui garantit des libellés stables entre installations.
pub trait SectorRepository: Send + Sync {
    /// List les secteurs dans l'ordre d'affichage.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<ActivitySector>>;
}
