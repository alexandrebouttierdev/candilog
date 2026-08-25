//! Helpers communs et déclaration des cas de test.
use super::*;

/// Relance de test, renvoyée par le dépôt double.
fn rel(date: &str, canal: &str) -> Relance {
    Relance {
        id: uuid::Uuid::nil(),
        candidature_id: uuid::Uuid::from_u128(1),
        candidature_poste: Some("Développeur Frontend".into()),
        entreprise_nom: Some("Nova Digital".into()),
        date_relance: date.into(),
        type_relance: canal.into(),
        notes: None,
        created_at: "2026-08-20T00:00:00Z".into(),
    }
}

/// Charge utile valide par défaut.
fn nouvelle(date: &str) -> NouvelleRelance {
    NouvelleRelance {
        candidature_id: uuid::Uuid::from_u128(1),
        date_relance: date.into(),
        type_relance: "Email".into(),
        notes: None,
    }
}

/// Dépôt double : renvoie ce qu'on lui donne, sans base.
struct StubRepo;

impl RelanceRepository for StubRepo {
    fn list(&self) -> AppResult<Vec<Relance>> {
        Ok(vec![])
    }
    fn list_between(&self, _from: &str, _to: &str) -> AppResult<Vec<Relance>> {
        Ok(vec![])
    }
    fn create(&self, input: &NouvelleRelance) -> AppResult<Relance> {
        Ok(rel(&input.date_relance, &input.type_relance))
    }
    fn update(&self, _id: uuid::Uuid, input: &NouvelleRelance) -> AppResult<Relance> {
        Ok(rel(&input.date_relance, &input.type_relance))
    }
    fn delete(&self, _id: uuid::Uuid) -> AppResult<()> {
        Ok(())
    }
}

mod test_canal_vide_est_refuse;
mod test_creer_valide_delegue_au_depot;
mod test_date_avec_heure_est_refusee;
