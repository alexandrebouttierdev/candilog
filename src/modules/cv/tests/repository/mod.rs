//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::shared::db::{open_pool, run_local_migrations};

fn repo() -> SqliteCvVersionRepository {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    SqliteCvVersionRepository::new(pool)
}

/// Vérifie qu'un contenu JSON imbriqué, avec tableaux et accents, fait l'aller-retour sans
/// altération : c'est le chemin qui casse en premier si le round-trip passe par une
/// désérialisation partielle ou un mauvais encodage.
mod test_create_puis_get_restitue_le_contenu_json;
mod test_create_puis_get_restitue_un_contenu_json_imbrique_et_accentue;
mod test_delete_supprime_la_version;
mod test_get_identifiant_inconnu_retourne_not_found;
mod test_list_ne_renvoie_que_les_resumes_les_plus_recents_d_abord;
