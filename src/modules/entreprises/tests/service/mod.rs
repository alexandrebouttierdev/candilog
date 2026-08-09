//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::modules::entreprises::model::Entreprise;

/// Construit une entreprise de test à partir d'un nom.
fn ent(nom: &str) -> Entreprise {
    Entreprise {
        id: uuid::Uuid::nil(),
        nom: nom.into(),
        secteur: None,
        type_: None,
        site_web: None,
        ville: None,
        adresse: None,
        notes: None,
        created_at: "now".into(),
        updated_at: "now".into(),
    }
}
/// Construit un payload de test à partir d'un nom.
fn nouvelle(nom: &str) -> NouvelleEntreprise {
    NouvelleEntreprise {
        nom: nom.into(),
        secteur: None,
        type_: None,
        site_web: None,
        ville: None,
        adresse: None,
        notes: None,
    }
}

struct StubRepo;
impl EntrepriseRepository for StubRepo {
    fn list(&self) -> AppResult<Vec<Entreprise>> {
        Ok(vec![])
    }
    fn create(&self, input: &NouvelleEntreprise) -> AppResult<Entreprise> {
        Ok(ent(&input.nom))
    }
    fn update(&self, _id: uuid::Uuid, input: &NouvelleEntreprise) -> AppResult<Entreprise> {
        Ok(ent(&input.nom))
    }
    fn delete(&self, _id: uuid::Uuid) -> AppResult<()> {
        Ok(())
    }
}

mod test_creer_nom_valide_delegue_au_depot;
mod test_creer_nom_vide_retourne_validation;
mod test_modifier_nom_valide_delegue_au_depot;
mod test_modifier_nom_vide_retourne_validation;
mod test_supprimer_delegue_au_depot;
