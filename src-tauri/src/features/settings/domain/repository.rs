//! Contract d'accès à la ligne singleton `parametres`.

use super::AppSettings;
use crate::core::errors::AppResult;

pub trait SettingsRepository: Send + Sync {
    fn get(&self) -> AppResult<AppSettings>;
    fn upsert(&self, settings: &AppSettings) -> AppResult<AppSettings>;
}
