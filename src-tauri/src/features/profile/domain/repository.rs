//! Contract de persistance du profil.

use crate::core::errors::AppResult;
use crate::features::profile::domain::Profile;

/// Accès à la ligne singleton du profil.
pub trait ProfileRepository: Send + Sync {
    /// Payload le profil et sa date de mise à jour, ou un profil vide si la ligne est absente.
    fn get(&self) -> AppResult<(Profile, Option<String>)>;

    /// Crée ou remplace la ligne unique et renvoie son horodatage.
    fn save(&self, profile: &Profile) -> AppResult<(Profile, String)>;
}
