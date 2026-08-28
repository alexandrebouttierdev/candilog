//! Contract d'accès aux agrégats d'analyse.

use crate::core::errors::AppResult;
use crate::features::analytics::domain::metrics::{
    ToFollowUp, UpcomingItem, Step, Metrics, Performance, ActivityWeek,
};
use crate::features::applications::domain::Application;

/// Accès aux agrégats calculés par la base.
///
/// Chaque méthode correspond à un bloc d'écran et est calculée **par `SQLite`**, jamais en
/// chargeant les lignes pour les compter en Rust : le guide interdit d'agréger dans la vue,
/// et agréger en mémoire reviendrait au même un étage plus bas.
pub trait AnalyticsRepository: Send + Sync {
    /// Metrics chiffrés depuis une date, ou sur tout l'historique.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn metrics(&self, from: Option<&str>) -> AppResult<Metrics>;

    /// Rythme et délais depuis une date.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn performance(&self, from: Option<&str>) -> AppResult<Performance>;

    /// Applications envoyées, groupées par semaine, sur les `semaines` dernières.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn activity_hebdomadaire(&self, semaines: u32) -> AppResult<Vec<ActivityWeek>>;

    /// Répartition du pipeline par statut courant.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn pipeline(&self) -> AppResult<Vec<Step>>;

    /// Prochains entretiens et relances, à partir d'aujourd'hui.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn upcoming_items(&self, today: &str, limite: u64) -> AppResult<Vec<UpcomingItem>>;

    /// Applications sans réponse depuis au moins `jours`, les plus anciennes d'abord.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn to_follow_up(&self, today: &str, days: u64, limite: u64) -> AppResult<Vec<ToFollowUp>>;

    /// Applications les plus récentes.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn recent(&self, limite: u64) -> AppResult<Vec<Application>>;
}
