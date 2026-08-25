//! Contrat d'accès au référentiel des secteurs.

use crate::core::errors::AppResult;
use crate::features::secteurs::domain::secteur::SecteurActivite;

/// Accès en lecture au référentiel des secteurs d'activité.
///
/// Le référentiel est en lecture seule pour l'application : il est alimenté au démarrage
/// depuis la liste canonique, jamais par l'utilisateur. Un secteur ne s'ajoute qu'en
/// modifiant [`SECTEURS_CANONIQUES`](crate::features::secteurs::infrastructure::SECTEURS_CANONIQUES),
/// ce qui garantit des libellés stables entre installations.
pub trait SecteurRepository: Send + Sync {
    /// Liste les secteurs dans l'ordre d'affichage.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn lister(&self) -> AppResult<Vec<SecteurActivite>>;
}
