//! Contrat d'accès aux agrégats d'analyse.

use crate::core::errors::AppResult;
use crate::features::analyses::domain::indicateurs::{
    ARelancer, Echeance, Etape, Indicateurs, Performance, SemaineActivite,
};
use crate::features::candidatures::domain::Candidature;

/// Accès aux agrégats calculés par la base.
///
/// Chaque méthode correspond à un bloc d'écran et est calculée **par `SQLite`**, jamais en
/// chargeant les lignes pour les compter en Rust : le guide interdit d'agréger dans la vue,
/// et agréger en mémoire reviendrait au même un étage plus bas.
pub trait AnalysesRepository: Send + Sync {
    /// Indicateurs chiffrés depuis une date, ou sur tout l'historique.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn indicateurs(&self, depuis: Option<&str>) -> AppResult<Indicateurs>;

    /// Rythme et délais depuis une date.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn performance(&self, depuis: Option<&str>) -> AppResult<Performance>;

    /// Candidatures envoyées, groupées par semaine, sur les `semaines` dernières.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn activite_hebdomadaire(&self, semaines: u32) -> AppResult<Vec<SemaineActivite>>;

    /// Répartition du pipeline par statut courant.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn pipeline(&self) -> AppResult<Vec<Etape>>;

    /// Prochains entretiens et relances, à partir d'aujourd'hui.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn echeances(&self, aujourdhui: &str, limite: u64) -> AppResult<Vec<Echeance>>;

    /// Candidatures sans réponse depuis au moins `jours`, les plus anciennes d'abord.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn a_relancer(&self, aujourdhui: &str, jours: u64, limite: u64) -> AppResult<Vec<ARelancer>>;

    /// Candidatures les plus récentes.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn recentes(&self, limite: u64) -> AppResult<Vec<Candidature>>;
}
