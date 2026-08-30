//! Contract d'accès aux entretiens.

use crate::core::errors::AppResult;
use crate::features::interviews::domain::interview::{Interview, InterviewAnalysis, NewInterview};
use uuid::Uuid;

/// Accès aux entretiens.
pub trait InterviewRepository: Send + Sync {
    /// List tous les entretiens, par date croissante.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<Interview>>;

    /// List les entretiens d'une plage de dates, bornes incluses.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list_between(&self, from: &str, to: &str) -> AppResult<Vec<Interview>>;

    /// Récupère un entretien par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn get(&self, id: Uuid) -> AppResult<Interview>;

    /// Enregistre l'entretien **et** fait passer sa candidature au statut « Interview ».
    ///
    /// Path unique de création et de modification : `id` absent crée, `id` présent
    /// modifie. L'application Iced exposait aussi un `create` et un `update` qui ne
    /// touchaient pas au statut — ils n'étaient appelés par aucun écran, et les migrer
    /// aurait conservé un piège : planifier un entretien sans que la candidature avance.
    ///
    /// # Errors
    /// `AppError::Validation` si la candidature liée est introuvable ;
    /// `AppError::NotFound` si `id` est fourni mais inconnu.
    fn save_and_mark_candidate(
        &self,
        id: Option<Uuid>,
        input: &NewInterview,
    ) -> AppResult<Interview>;

    /// Supprime un entretien.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu ; `AppError::Database` si la
    /// suppression échoue.
    fn delete(&self, id: Uuid) -> AppResult<()>;

    /// Enregistre l'analyse `IA` du compte rendu.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn save_analysis(&self, id: Uuid, analysis: &InterviewAnalysis) -> AppResult<()>;
}
