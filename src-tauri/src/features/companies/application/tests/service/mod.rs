//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::features::companies::domain::{Company, CompanySize};

/// Entreprise de test, renvoyée par le dépôt double.
fn ent(name: &str) -> Company {
    Company {
        id: uuid::Uuid::nil(),
        name: name.into(),
        sector_id: None,
        sector_name: None,
        company_type_id: None,
        company_type_name: None,
        company_size: CompanySize::Unknown,
        website: None,
        city: None,
        address: None,
        notes: None,
        created_at: "now".into(),
        updated_at: "now".into(),
    }
}

/// Payload utile de test, avec le seul champ requis.
fn new(name: &str) -> NewCompany {
    NewCompany {
        name: name.into(),
        sector_id: None,
        company_type_id: None,
        company_size: CompanySize::Unknown,
        website: None,
        city: None,
        address: None,
        notes: None,
    }
}

/// Dépôt double : renvoie ce qu'on lui donne, sans base.
///
/// Ce que ces tests éprouvent est la validation du service, pas le SQL — les requêtes sont
/// couvertes par les tests du dépôt, sur une base en mémoire.
struct StubRepo;

impl CompanyRepository for StubRepo {
    fn list(&self) -> AppResult<Vec<Company>> {
        Ok(vec![])
    }
    fn get(&self, _id: uuid::Uuid) -> AppResult<Company> {
        Ok(ent("Nova Digital"))
    }
    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        _filter: &CompanyFilter,
    ) -> AppResult<crate::core::pagination::Page<Company>> {
        Ok(crate::core::pagination::Page::new(
            vec![],
            0,
            page,
            page_size,
        ))
    }
    fn create(&self, input: &NewCompany) -> AppResult<Company> {
        Ok(ent(&input.name))
    }
    fn update(&self, _id: uuid::Uuid, input: &NewCompany) -> AppResult<Company> {
        Ok(ent(&input.name))
    }
    fn delete(&self, _id: uuid::Uuid) -> AppResult<()> {
        Ok(())
    }
}

mod test_creer_nom_valide_delegue_au_depot;
mod test_creer_nom_vide_retourne_validation;
mod test_creer_refuse_un_site_web_non_http;
mod test_modifier_nom_valide_delegue_au_depot;
mod test_modifier_nom_vide_retourne_validation;
mod test_supprimer_delegue_au_depot;
