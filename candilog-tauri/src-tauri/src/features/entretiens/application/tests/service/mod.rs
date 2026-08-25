//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::features::entretiens::domain::TypeEntretien;

/// Entretien de test, renvoyé par le dépôt double.
fn ent(date: &str) -> Entretien {
    Entretien {
        id: uuid::Uuid::nil(),
        candidature_id: uuid::Uuid::from_u128(1),
        candidature_poste: Some("Développeur Frontend".into()),
        entreprise_nom: Some("Nova Digital".into()),
        contact_id: None,
        contact_nom: None,
        date_entretien: date.into(),
        type_entretien: TypeEntretien::Visio,
        lieu: None,
        notes: None,
        compte_rendu: None,
        analyse_ia: None,
        created_at: "2026-08-20T00:00:00Z".into(),
        updated_at: "2026-08-20T00:00:00Z".into(),
    }
}

/// Charge utile valide par défaut.
fn nouvel(date: &str) -> NouvelEntretien {
    NouvelEntretien {
        candidature_id: uuid::Uuid::from_u128(1),
        contact_id: None,
        date_entretien: date.into(),
        type_entretien: TypeEntretien::Visio,
        lieu: None,
        notes: None,
        compte_rendu: None,
    }
}

/// Dépôt double : renvoie ce qu'on lui donne, sans base.
struct StubRepo;

impl EntretienRepository for StubRepo {
    fn list(&self) -> AppResult<Vec<Entretien>> {
        Ok(vec![])
    }
    fn list_between(&self, _from: &str, _to: &str) -> AppResult<Vec<Entretien>> {
        Ok(vec![])
    }
    fn get(&self, _id: uuid::Uuid) -> AppResult<Entretien> {
        Ok(ent("2026-08-25T14:00:00+02:00"))
    }
    fn save_and_mark_candidate(
        &self,
        _id: Option<uuid::Uuid>,
        input: &NouvelEntretien,
    ) -> AppResult<Entretien> {
        Ok(ent(&input.date_entretien))
    }
    fn delete(&self, _id: uuid::Uuid) -> AppResult<()> {
        Ok(())
    }
    fn enregistrer_analyse(&self, _id: uuid::Uuid, _analyse: &AnalyseEntretien) -> AppResult<()> {
        Ok(())
    }
}

mod test_candidature_nulle_est_refusee;
mod test_date_sans_heure_est_refusee;
mod test_enregistrer_valide_delegue_au_depot;
