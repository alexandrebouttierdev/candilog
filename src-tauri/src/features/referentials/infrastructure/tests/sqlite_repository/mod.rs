//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::core::database::helpers::connection;
use crate::core::database::{open_pool, run_local_migrations, SqlitePool};

/// Dépôt sur base mémoire, schéma et semences appliqués.
fn context() -> (SqliteReferentialRepository, SqlitePool) {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    (SqliteReferentialRepository::new(pool.clone()), pool)
}

fn repo() -> SqliteReferentialRepository {
    context().0
}

/// Libellé associé à un code, ou `None` si le code est absent du catalogue.
fn label(items: &[ReferenceItem], code: &str) -> Option<String> {
    items
        .iter()
        .find(|item| item.code == code)
        .map(|item| item.name.clone())
}

/// Rejoue `init_schema.sql` sur la base déjà initialisée.
///
/// Le runner de migrations s'arrête au curseur `user_version` : pour vérifier que les
/// semences sont bien idempotentes, il faut rejouer le fichier lui-même.
fn replay_schema(pool: &SqlitePool) {
    let conn = connection(pool).unwrap();
    conn.execute_batch(include_str!("../../../../../../migrations/init_schema.sql"))
        .unwrap();
}

mod test_les_domaines_professionnels_sont_complets_et_ordonnes;
mod test_les_secteurs_restent_distincts_des_domaines_professionnels;
mod test_les_semences_rejouees_ne_dupliquent_rien;
mod test_les_types_d_entreprise_sont_complets;
mod test_les_types_de_contrat_sont_complets;
