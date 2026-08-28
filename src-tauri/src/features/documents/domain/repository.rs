//! Contrats d'accès aux bibliothèques locales.

use super::{CvResume, CvVersion, Lettre, NouveauCv, NouvelleLettre};
use crate::core::errors::AppResult;
use uuid::Uuid;

pub trait CvRepository: Send + Sync {
    fn enregistrer(&self, input: &NouveauCv) -> AppResult<CvVersion>;
    fn lister(&self) -> AppResult<Vec<CvResume>>;
    fn obtenir(&self, id: Uuid) -> AppResult<CvVersion>;
    fn supprimer(&self, id: Uuid) -> AppResult<()>;
}

pub trait LettreRepository: Send + Sync {
    fn enregistrer(&self, input: &NouvelleLettre) -> AppResult<Lettre>;
    fn lister(&self) -> AppResult<Vec<Lettre>>;
    fn obtenir(&self, id: Uuid) -> AppResult<Lettre>;
    fn supprimer(&self, id: Uuid) -> AppResult<()>;
}
