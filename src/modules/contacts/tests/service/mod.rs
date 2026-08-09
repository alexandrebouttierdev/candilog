//! Helpers communs et déclaration des cas de test.
use super::*;

/// Construit un contact de test.
fn ct(prenom: &str, nom: &str) -> Contact {
    Contact {
        id: uuid::Uuid::nil(),
        entreprise_id: None,
        prenom: prenom.into(),
        nom: nom.into(),
        poste: None,
        email: None,
        telephone: None,
        linkedin: None,
        notes: None,
        created_at: "now".into(),
        updated_at: "now".into(),
    }
}
/// Construit un payload de test.
fn nouveau(prenom: &str, nom: &str) -> NouveauContact {
    NouveauContact {
        entreprise_id: None,
        prenom: prenom.into(),
        nom: nom.into(),
        poste: None,
        email: None,
        telephone: None,
        linkedin: None,
        notes: None,
    }
}

struct StubRepo;
impl ContactRepository for StubRepo {
    fn list(&self) -> AppResult<Vec<Contact>> {
        Ok(vec![])
    }
    fn create(&self, input: &NouveauContact) -> AppResult<Contact> {
        Ok(ct(&input.prenom, &input.nom))
    }
    fn update(&self, _id: uuid::Uuid, input: &NouveauContact) -> AppResult<Contact> {
        Ok(ct(&input.prenom, &input.nom))
    }
    fn delete(&self, _id: uuid::Uuid) -> AppResult<()> {
        Ok(())
    }
}

mod test_creer_nom_vide_retourne_validation;
mod test_creer_prenom_vide_retourne_validation;
mod test_creer_valide_delegue_au_depot;
mod test_modifier_valide_delegue_au_depot;
mod test_supprimer_delegue_au_depot;
