//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::shared::db::{open_pool, run_local_migrations};

fn repo() -> SqliteRelanceRepository {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    SqliteRelanceRepository::new(pool)
}

fn candidature(repo: &SqliteRelanceRepository) -> uuid::Uuid {
    let entreprise = uuid::Uuid::new_v4();
    let candidature = uuid::Uuid::new_v4();
    let conn = crate::shared::sqlite::connexion(&repo.pool).unwrap();
    conn.execute(
        "INSERT INTO entreprises (id, nom, created_at, updated_at)
             VALUES (?1, 'ACME', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [entreprise.to_string()],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO candidatures (id, entreprise_id, poste, date_envoi, created_at, updated_at)
             VALUES (?1, ?2, 'Dev', '2026-01-01', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        rusqlite::params![candidature.to_string(), entreprise.to_string()],
    )
    .unwrap();
    candidature
}

fn entree(candidature_id: uuid::Uuid, date: &str) -> NouvelleRelance {
    NouvelleRelance {
        candidature_id,
        date_relance: date.into(),
        type_relance: "Email".into(),
        notes: None,
    }
}

mod test_create_candidature_inconnue_retourne_validation;
mod test_create_puis_list_restitue_la_relance;
mod test_delete_supprime_la_relance;
mod test_list_trie_par_date_croissante;
mod test_update_candidature_inconnue_retourne_validation;
mod test_update_identifiant_inconnu_retourne_not_found;
mod test_update_modifie_la_date_et_les_notes;
