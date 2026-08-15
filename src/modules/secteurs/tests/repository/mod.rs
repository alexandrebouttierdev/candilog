//! Helpers communs et déclaration des cas de test du dépôt des secteurs.
use super::*;
use crate::shared::db::{open_pool, run_local_migrations};

fn pool() -> crate::shared::db::SqlitePool {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    pool
}

mod test_garantir_referentiel_ajoute_la_liste_canonique;
mod test_garantir_referentiel_est_idempotent;
mod test_garantir_referentiel_rattache_les_valeurs_libres_existantes;
