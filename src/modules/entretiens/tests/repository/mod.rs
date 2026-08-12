//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::modules::entretiens::model::TypeEntretien;
use crate::shared::db::{open_pool, run_local_migrations};

fn repo() -> SqliteEntretienRepository {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    SqliteEntretienRepository::new(pool)
}

fn candidature(repo: &SqliteEntretienRepository) -> uuid::Uuid {
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

fn entree(candidature_id: uuid::Uuid, date: &str) -> NouvelEntretien {
    NouvelEntretien {
        candidature_id,
        contact_id: None,
        date_entretien: date.into(),
        type_entretien: TypeEntretien::Visio,
        lieu: Some("Google Meet".into()),
        notes: None,
        compte_rendu: None,
    }
}

mod test_create_puis_get_restitue_le_type_et_le_lieu;
mod test_delete_supprime_l_entretien;
mod test_enregistrer_analyse_puis_get_restitue_l_analyse;
mod test_get_identifiant_inconnu_retourne_not_found;
mod test_list_trie_par_date_croissante;
mod test_save_and_mark_candidate_est_atomique;
mod test_update_modifie_le_compte_rendu;
mod test_update_preserve_l_analyse_ia_enregistree;
