//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::shared::db::{open_pool, run_local_migrations};

fn repo() -> SqliteCacheIaRepository {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    SqliteCacheIaRepository::new(pool)
}

fn entry(cle: &str, valeur: &str) -> CacheEntry {
    CacheEntry {
        cle: cle.into(),
        valeur: valeur.into(),
        provider: "ollama".into(),
        modele: "gemma3:1b".into(),
        operation: "parse_cv".into(),
        cree_le: "2026-07-25T00:00:00Z".into(),
    }
}

mod test_cache_key_stable_et_sensible_aux_entrees;
mod test_get_absent_retourne_none;
mod test_put_puis_get_retourne_la_valeur;
mod test_put_remplace_une_cle_existante;
mod test_reset_vide_le_cache;
