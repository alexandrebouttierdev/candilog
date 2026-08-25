//! Helpers communs et déclaration des cas de test.
use super::*;

/// Amène une connexion neuve à l'état d'une base `user_version = 4`, c'est-à-dire juste
/// avant la migration 005. Permet de peupler des données métier puis de migrer.
#[cfg(test)]
fn preparer_base_version_quatre(conn: &rusqlite::Connection) {
    for (cible, sql) in MIGRATIONS {
        if *cible > 4 {
            break;
        }
        conn.execute_batch(sql).unwrap();
    }
    conn.pragma_update(None, "user_version", 4_i64).unwrap();
}

/// Nombre de lignes d'une table. `table` provient exclusivement de littéraux de test.
#[cfg(test)]
fn compter(conn: &rusqlite::Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap()
}

mod test_connexion_du_pool_active_les_cles_etrangeres;
mod test_migration_005_conserve_les_valeurs_d_enum_heritees;
mod test_migration_005_preserve_les_donnees_metier_liees;
mod test_migration_006_indexe_les_dates;
mod test_migration_008_cree_le_referentiel_des_secteurs;
mod test_migration_metier_cree_toutes_les_tables;
mod test_migrations_base_heritee_ne_rejoue_pas_les_versions_deja_appliquees;
mod test_migrations_base_heritee_version_zero_purge_et_supprime_les_tables_retirees;
mod test_migrations_base_neuve_applique_toutes_les_versions;
mod test_migrations_executees_deux_fois_restent_idempotentes;
mod test_migrations_rendent_la_connexion_cles_etrangeres_actives;
mod test_migrations_sont_declarees_dans_un_ordre_strictement_croissant;
mod test_open_pool_abandonne_vite_sur_une_base_illisible;
mod test_open_pool_memoire_partage_les_donnees_entre_connexions;
mod test_suppression_candidature_efface_ses_relances_en_cascade;
mod test_suppression_entreprise_referencee_est_refusee;
mod test_table_parametres_refuse_une_seconde_ligne;
