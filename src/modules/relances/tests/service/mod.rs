//! Helpers communs et déclaration des cas de test.
use super::*;

fn rel() -> Relance {
    Relance {
        id: uuid::Uuid::nil(),
        candidature_id: uuid::Uuid::from_u128(1),
        date_relance: "2026-07-14T10:00:00Z".into(),
        type_relance: "Email".into(),
        notes: None,
        created_at: "now".into(),
    }
}
fn nouvelle(cid: u128, date: &str) -> NouvelleRelance {
    NouvelleRelance {
        candidature_id: uuid::Uuid::from_u128(cid),
        date_relance: date.into(),
        type_relance: "Email".into(),
        notes: None,
    }
}

struct StubRepo;
impl RelanceRepository for StubRepo {
    fn list(&self) -> AppResult<Vec<Relance>> {
        Ok(vec![rel()])
    }
    fn create(&self, _i: &NouvelleRelance) -> AppResult<Relance> {
        Ok(rel())
    }
    fn update(&self, _id: uuid::Uuid, _i: &NouvelleRelance) -> AppResult<Relance> {
        Ok(rel())
    }
    fn delete(&self, _id: uuid::Uuid) -> AppResult<()> {
        Ok(())
    }
}

mod test_creer_candidature_nulle_retourne_validation;
mod test_creer_date_vide_retourne_validation;
mod test_creer_valide_delegue_au_depot;
mod test_lister_delegue_au_depot;
mod test_supprimer_delegue_au_depot;
