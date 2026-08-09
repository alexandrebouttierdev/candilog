//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::shared::db::{open_pool, run_local_migrations};

fn repo() -> SqliteSettingsRepository {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    SqliteSettingsRepository::new(pool)
}

mod test_get_parametres_illisibles_retourne_les_valeurs_par_defaut;
/// Une configuration non triviale (fournisseur cloud avec clé, thème et langue modifiés)
/// doit faire l'aller-retour `upsert` → `get` sans aucune altération : c'est le chemin qui
/// casse en premier si la sérialisation JSON tronque un champ optionnel.
mod test_get_sans_parametres_enregistres_retourne_les_valeurs_par_defaut;
mod test_upsert_appele_deux_fois_ne_cree_pas_de_seconde_ligne;
mod test_upsert_puis_get_restitue_des_parametres_non_triviaux_sans_alteration;
mod test_upsert_puis_get_restitue_les_parametres;
