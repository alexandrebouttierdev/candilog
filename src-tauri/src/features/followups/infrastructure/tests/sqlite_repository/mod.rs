//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::core::database::{open_pool, run_local_migrations};

/// Dépôt sur base mémoire migrée, avec une candidature déjà créée.
fn context() -> (SqliteFollowUpRepository, Uuid) {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = connection(&pool).unwrap();
    let company_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO companies (id, name, created_at, updated_at)
         VALUES (?1, 'Atlas Studio', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [company_id.to_string()],
    )
    .unwrap();
    let application_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO applications (id, company_id, job_title, contract_type, status, sent_date,
            created_at, updated_at)
         VALUES (?1, ?2, 'Product Designer', 'CDI', 'EN_ATTENTE', '2026-08-10',
            '2026-08-10T00:00:00Z', '2026-08-10T00:00:00Z')",
        [application_id.to_string(), company_id.to_string()],
    )
    .unwrap();
    drop(conn);
    (SqliteFollowUpRepository::new(pool), application_id)
}

/// Payload utile valide, dont seule la date varie selon les tests.
fn entree(application_id: Uuid, date: &str) -> NewFollowUp {
    NewFollowUp {
        application_id,
        follow_up_date: date.into(),
        channel: "Email".into(),
        notes: None,
    }
}

/// Status courant d'une candidature.
fn status(repo: &SqliteFollowUpRepository, application_id: Uuid) -> String {
    connection(&repo.pool)
        .unwrap()
        .query_row(
            "SELECT status FROM applications WHERE id = ?1",
            [application_id.to_string()],
            |row| row.get(0),
        )
        .unwrap()
}

mod test_create_ne_touche_pas_au_statut_de_la_candidature;
mod test_create_sur_candidature_inconnue_retourne_une_phrase_lisible;
mod test_delete_identifiant_inconnu_retourne_not_found;
mod test_la_plage_du_calendrier_inclut_ses_bornes;
mod test_le_poste_et_l_entreprise_sont_aplatis;
mod test_update_identifiant_inconnu_retourne_not_found;
