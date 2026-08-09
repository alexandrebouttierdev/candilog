//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::modules::entretiens::model::TypeEntretien;

fn ent() -> Entretien {
    Entretien {
        id: uuid::Uuid::nil(),
        candidature_id: uuid::Uuid::from_u128(1),
        contact_id: None,
        date_entretien: "2026-07-20T09:00:00Z".into(),
        type_entretien: TypeEntretien::Presentiel,
        lieu: None,
        notes: None,
        compte_rendu: None,
        analyse_ia: None,
        created_at: "now".into(),
        updated_at: "now".into(),
    }
}
fn nouveau(cid: u128, date: &str) -> NouvelEntretien {
    NouvelEntretien {
        candidature_id: uuid::Uuid::from_u128(cid),
        contact_id: None,
        date_entretien: date.into(),
        type_entretien: TypeEntretien::Presentiel,
        lieu: None,
        notes: None,
        compte_rendu: None,
    }
}

struct StubRepo;
impl EntretienRepository for StubRepo {
    fn list(&self) -> AppResult<Vec<Entretien>> {
        Ok(vec![ent()])
    }
    fn create(&self, _i: &NouvelEntretien) -> AppResult<Entretien> {
        Ok(ent())
    }
    fn update(&self, _id: uuid::Uuid, _i: &NouvelEntretien) -> AppResult<Entretien> {
        Ok(ent())
    }
    fn delete(&self, _id: uuid::Uuid) -> AppResult<()> {
        Ok(())
    }
    fn get(&self, _id: uuid::Uuid) -> AppResult<Entretien> {
        Ok(ent())
    }
    fn enregistrer_analyse(&self, _id: uuid::Uuid, _analyse: &AnalyseEntretien) -> AppResult<()> {
        Ok(())
    }
}

mod test_creer_candidature_nulle_retourne_validation;
mod test_creer_date_vide_retourne_validation;
mod test_creer_valide_delegue_au_depot;
mod test_enregistrer_analyse_delegue_au_depot;
mod test_lister_delegue_au_depot;
mod test_obtenir_delegue_au_depot;
mod test_supprimer_delegue_au_depot;
