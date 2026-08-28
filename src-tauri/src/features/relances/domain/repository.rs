//! Contrat d'accès aux relances.

use crate::core::errors::AppResult;
use crate::features::relances::domain::relance::{NouvelleRelance, Relance};
use uuid::Uuid;

/// Accès aux relances.
pub trait RelanceRepository: Send + Sync {
    /// Liste toutes les relances, par date décroissante.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<Relance>>;

    /// Liste les relances d'une plage de dates, bornes incluses.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list_between(&self, from: &str, to: &str) -> AppResult<Vec<Relance>>;

    /// Crée une relance.
    ///
    /// Ne touche **pas** au statut de la candidature, contrairement à l'enregistrement d'un
    /// entretien. C'est le comportement de l'application Iced, conservé tel quel : le statut
    /// « Relancée » reste posé à la main. L'asymétrie est signalée dans
    /// `docs/migration/02-JOURNAL.md` — la corriger serait un changement de comportement,
    /// pas une migration.
    ///
    /// # Errors
    /// `AppError::Validation` si la candidature liée est introuvable.
    fn create(&self, input: &NouvelleRelance) -> AppResult<Relance>;

    /// Remplace les champs d'une relance.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn update(&self, id: Uuid, input: &NouvelleRelance) -> AppResult<Relance>;

    /// Supprime une relance.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la suppression échoue.
    fn delete(&self, id: Uuid) -> AppResult<()>;
}
