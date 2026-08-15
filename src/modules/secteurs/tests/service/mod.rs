//! Helpers communs et déclaration des cas de test du service des secteurs.
use super::*;
use crate::modules::secteurs::model::SecteurActivite;
use crate::modules::secteurs::repository::SecteurRepository;
use crate::shared::error::AppResult;

struct StubRepo;
impl SecteurRepository for StubRepo {
    fn lister(&self) -> AppResult<Vec<SecteurActivite>> {
        Ok(vec![SecteurActivite {
            id: uuid::Uuid::nil(),
            nom: "Santé".into(),
        }])
    }
}

mod test_lister_delegue_au_depot;
