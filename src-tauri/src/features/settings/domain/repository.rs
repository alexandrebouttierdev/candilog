//! Contract d'accès à la ligne singleton `parametres`.

use super::AppSettings;
use crate::core::errors::AppResult;

pub trait SettingsRepository: Send + Sync {
    fn get(&self) -> AppResult<AppSettings>;
    fn upsert(&self, settings: &AppSettings) -> AppResult<AppSettings>;

    /// Instant RFC 3339 de la dernière interrogation de l'API GitHub, quelle qu'en ait été
    /// l'issue, ou `None` si l'application ne l'a jamais tentée.
    ///
    /// Ce n'est pas un paramètre utilisateur : rien ne le règle, rien ne l'affiche. Il ne
    /// vit donc pas dans `AppSettings` mais à côté, pour que la vérification au démarrage
    /// sache si le délai est écoulé sans interroger GitHub pour le savoir.
    ///
    /// # Errors
    /// Retourne une erreur si la lecture échoue.
    fn last_update_check(&self) -> AppResult<Option<String>>;

    /// Retient l'instant d'une tentative de vérification.
    ///
    /// # Errors
    /// Retourne une erreur si l'écriture échoue.
    fn mark_update_check(&self, instant: &str) -> AppResult<()>;
}
