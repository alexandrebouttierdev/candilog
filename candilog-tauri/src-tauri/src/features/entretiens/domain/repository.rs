//! Contrat d'accès aux entretiens.

use crate::core::errors::AppResult;
use crate::features::entretiens::domain::entretien::{
    AnalyseEntretien, Entretien, NouvelEntretien,
};
use uuid::Uuid;

/// Accès aux entretiens.
pub trait EntretienRepository: Send + Sync {
    /// Liste tous les entretiens, par date croissante.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list(&self) -> AppResult<Vec<Entretien>>;

    /// Liste les entretiens d'une plage de dates, bornes incluses.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la requête échoue.
    fn list_between(&self, from: &str, to: &str) -> AppResult<Vec<Entretien>>;

    /// Récupère un entretien par identifiant.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn get(&self, id: Uuid) -> AppResult<Entretien>;

    /// Enregistre l'entretien **et** fait passer sa candidature au statut « Entretien ».
    ///
    /// Chemin unique de création et de modification : `id` absent crée, `id` présent
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
        input: &NouvelEntretien,
    ) -> AppResult<Entretien>;

    /// Supprime un entretien.
    ///
    /// # Errors
    /// Retourne `AppError::Database` si la suppression échoue.
    fn delete(&self, id: Uuid) -> AppResult<()>;

    /// Enregistre l'analyse `IA` du compte rendu.
    ///
    /// # Errors
    /// `AppError::NotFound` si l'identifiant est inconnu.
    fn enregistrer_analyse(&self, id: Uuid, analyse: &AnalyseEntretien) -> AppResult<()>;
}
