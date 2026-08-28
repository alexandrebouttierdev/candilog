//! Contrat d'accès à la ligne singleton `parametres`.

use super::AppSettings;
use crate::core::errors::AppResult;

pub trait ParametresRepository: Send + Sync {
    fn get(&self) -> AppResult<AppSettings>;
    fn upsert(&self, settings: &AppSettings) -> AppResult<AppSettings>;
}
