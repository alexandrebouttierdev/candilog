//! Contrat de persistance du profil.

use crate::core::errors::AppResult;
use crate::features::profil::domain::Profil;

/// Accès à la ligne singleton du profil.
pub trait ProfilRepository: Send + Sync {
    /// Charge le profil et sa date de mise à jour, ou un profil vide si la ligne est absente.
    fn obtenir(&self) -> AppResult<(Profil, Option<String>)>;

    /// Crée ou remplace la ligne unique et renvoie son horodatage.
    fn enregistrer(&self, profil: &Profil) -> AppResult<(Profil, String)>;
}
