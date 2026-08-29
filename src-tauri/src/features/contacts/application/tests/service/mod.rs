//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::features::contacts::domain::Contact;

/// Contact de test, renvoyé par le dépôt double.
fn ct(first_name: &str, name: &str) -> Contact {
    Contact {
        id: uuid::Uuid::nil(),
        company_id: None,
        company_name: None,
        first_name: first_name.into(),
        name: name.into(),
        job_title: None,
        tracking_role: None,
        email: None,
        phone: None,
        linkedin: None,
        notes: None,
        created_at: "now".into(),
        updated_at: "now".into(),
    }
}

/// Payload utile de test, avec les seuls champs requis.
fn new(first_name: &str, name: &str) -> NewContact {
    NewContact {
        company_id: None,
        first_name: first_name.into(),
        name: name.into(),
        job_title: None,
        tracking_role: None,
        email: None,
        phone: None,
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
        _tracking_role: Option<&str>,
    ) -> AppResult<crate::core::pagination::Page<Contact>> {
        Ok(crate::core::pagination::Page::new(
            vec![],
            0,
            page,
            page_size,
        ))
    }
    fn create(&self, input: &NewContact) -> AppResult<Contact> {
        Ok(ct(&input.first_name, &input.name))
    }
    fn update(&self, _id: uuid::Uuid, input: &NewContact) -> AppResult<Contact> {
        Ok(ct(&input.first_name, &input.name))
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
