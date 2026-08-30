//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::core::database::{open_pool, run_local_migrations};
use crate::features::companies::domain::CompanySize;

/// Identifiant du secteur « Informatique / Télécommunication », semé par `init_schema.sql`.
const SECTEUR_INFORMATIQUE: &str = "5ec70000-0000-4000-8000-00000000000d";

fn repo() -> SqliteCompanyRepository {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    SqliteCompanyRepository::new(pool)
}

fn entree(name: &str) -> NewCompany {
    NewCompany {
        name: name.into(),
        sector_id: None,
        company_type_id: Some("IT_SERVICES_COMPANY".into()),
        company_size: CompanySize::Pme,
        website: None,
        city: Some("Lyon".into()),
        address: None,
        notes: None,
    }
}

/// Filtre de recherche libre, sans autre critère.
fn recherche(search: &str) -> CompanyFilter {
    CompanyFilter {
        search: search.into(),
        ..CompanyFilter::default()
    }
}

mod test_create_puis_list_restitue_le_secteur_lie;
mod test_create_puis_list_restitue_les_champs;
mod test_delete_entreprise_avec_candidature_retourne_validation;
mod test_delete_identifiant_inconnu_retourne_not_found;
mod test_delete_supprime_l_entreprise;
mod test_la_recherche_ignore_les_accents_et_la_casse;
mod test_le_type_et_la_taille_restent_deux_dimensions;
mod test_list_trie_par_nom_croissant;
mod test_pagination_accede_aux_elements_apres_deux_cents;
mod test_pagination_applique_la_recherche_avant_la_limite;
mod test_pagination_filtre_par_type_avant_la_limite;
mod test_update_identifiant_inconnu_retourne_not_found;
mod test_update_modifie_les_champs_et_l_horodatage;
