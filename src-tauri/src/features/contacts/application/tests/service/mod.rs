//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::features::contacts::domain::Contact;

/// Contact de test, renvoyé par le dépôt double.
fn ct(prenom: &str, nom: &str) -> Contact {
    Contact {
        id: uuid::Uuid::nil(),
        entreprise_id: None,
        entreprise_nom: None,
        prenom: prenom.into(),
        nom: nom.into(),
        poste: None,
        role_suivi: None,
        email: None,
        telephone: None,
        linkedin: None,
        notes: None,
        created_at: "now".into(),
        updated_at: "now".into(),
    }
}

/// Charge utile de test, avec les seuls champs requis.
fn nouveau(prenom: &str, nom: &str) -> NouveauContact {
    NouveauContact {
        entreprise_id: None,
        prenom: prenom.into(),
        nom: nom.into(),
        poste: None,
        role_suivi: None,
        email: None,
        telephone: None,
        linkedin: None,
        notes: None,
    }
}

/// Dépôt double : renvoie ce qu'on lui donne, sans base.
///
/// Ce que ces tests éprouvent est la validation du service, pas le SQL — les requêtes sont
/// couvertes par les tests du dépôt, sur une base en mémoire.
struct StubRepo;

impl ContactRepository for StubRepo {
    fn list(&self) -> AppResult<Vec<Contact>> {
        Ok(vec![])
    }
    fn get(&self, _id: uuid::Uuid) -> AppResult<Contact> {
        Ok(ct("Camille", "Rivet"))
    }
    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        _search: &str,
    ) -> AppResult<crate::core::pagination::Page<Contact>> {
        Ok(crate::core::pagination::Page::new(
            vec![],
            0,
            page,
            page_size,
        ))
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
mod test_creer_refuse_un_linkedin_non_http;
mod test_creer_valide_delegue_au_depot;
mod test_modifier_valide_delegue_au_depot;
mod test_supprimer_delegue_au_depot;
