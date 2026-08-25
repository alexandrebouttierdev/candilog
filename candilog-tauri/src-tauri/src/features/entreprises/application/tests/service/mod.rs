//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::features::entreprises::domain::Entreprise;

/// Entreprise de test, renvoyée par le dépôt double.
fn ent(nom: &str) -> Entreprise {
    Entreprise {
        id: uuid::Uuid::nil(),
        nom: nom.into(),
        secteur_id: None,
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

/// Charge utile de test, avec le seul champ requis.
fn nouvelle(nom: &str) -> NouvelleEntreprise {
    NouvelleEntreprise {
        nom: nom.into(),
        secteur_id: None,
        secteur: None,
        type_: None,
        site_web: None,
        ville: None,
        adresse: None,
        notes: None,
    }
}

/// Dépôt double : renvoie ce qu'on lui donne, sans base.
///
/// Ce que ces tests éprouvent est la validation du service, pas le SQL — les requêtes sont
/// couvertes par les tests du dépôt, sur une base en mémoire.
struct StubRepo;

impl EntrepriseRepository for StubRepo {
    fn list(&self) -> AppResult<Vec<Entreprise>> {
        Ok(vec![])
    }
    fn get(&self, _id: uuid::Uuid) -> AppResult<Entreprise> {
        Ok(ent("Nova Digital"))
    }
    fn list_page(
        &self,
        page: u64,
        page_size: u64,
        _search: &str,
        _company_type: Option<&str>,
    ) -> AppResult<crate::core::pagination::Page<Entreprise>> {
        Ok(crate::core::pagination::Page::new(
            vec![],
            0,
            page,
            page_size,
        ))
    }
    fn list_types(&self) -> AppResult<Vec<String>> {
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
mod test_creer_refuse_un_site_web_non_http;
mod test_modifier_nom_valide_delegue_au_depot;
mod test_modifier_nom_vide_retourne_validation;
mod test_supprimer_delegue_au_depot;
