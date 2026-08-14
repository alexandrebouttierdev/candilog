//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::shared::db::{open_pool, run_local_migrations};

fn repo() -> SqliteEntrepriseRepository {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    SqliteEntrepriseRepository::new(pool)
}

fn entree(nom: &str) -> NouvelleEntreprise {
    NouvelleEntreprise {
        nom: nom.into(),
        secteur: Some("Tech".into()),
        type_: Some("ESN".into()),
        site_web: None,
        ville: Some("Lyon".into()),
        adresse: None,
        notes: None,
    }
}

mod test_create_puis_list_restitue_les_champs;
mod test_delete_entreprise_avec_candidature_retourne_validation;
mod test_delete_supprime_l_entreprise;
mod test_list_trie_par_nom_croissant;
mod test_pagination_accede_aux_elements_apres_deux_cents;
mod test_pagination_applique_la_recherche_avant_la_limite;
mod test_pagination_filtre_par_type_avant_la_limite;
mod test_update_identifiant_inconnu_retourne_not_found;
mod test_update_modifie_les_champs_et_l_horodatage;
