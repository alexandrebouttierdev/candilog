//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::shared::db::{open_pool, run_local_migrations};

fn repo() -> SqliteProfilRepository {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    SqliteProfilRepository::new(pool)
}

/// Un profil riche (expérience, compétence, caractère accentué) doit faire l'aller-retour
/// `upsert` → `get` sans aucune altération : c'est le chemin qui casse en premier si la
/// sérialisation JSON tronque un champ optionnel ou mal encode l'UTF-8.
mod test_get_sans_profil_enregistre_retourne_le_profil_par_defaut;
mod test_upsert_appele_deux_fois_ecrase_sans_creer_de_seconde_ligne;
mod test_upsert_puis_get_restitue_le_profil;
mod test_upsert_puis_get_restitue_un_profil_riche_sans_alteration;
