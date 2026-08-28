//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::core::database::{open_pool, run_local_migrations};

/// Dépôt sur base mémoire migrée, avec une candidature déjà créée.
fn contexte() -> (SqliteRelanceRepository, Uuid) {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = connexion(&pool).unwrap();
    let entreprise_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO entreprises (id, nom, created_at, updated_at)
         VALUES (?1, 'Atlas Studio', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [entreprise_id.to_string()],
    )
    .unwrap();
    let candidature_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO candidatures (id, entreprise_id, poste, type_contrat, statut, date_envoi,
            created_at, updated_at)
         VALUES (?1, ?2, 'Product Designer', 'CDI', 'EN_ATTENTE', '2026-08-10',
            '2026-08-10T00:00:00Z', '2026-08-10T00:00:00Z')",
        [candidature_id.to_string(), entreprise_id.to_string()],
    )
    .unwrap();
    drop(conn);
    (SqliteRelanceRepository::new(pool), candidature_id)
}

/// Charge utile valide, dont seule la date varie selon les tests.
fn entree(candidature_id: Uuid, date: &str) -> NouvelleRelance {
    NouvelleRelance {
        candidature_id,
        date_relance: date.into(),
        type_relance: "Email".into(),
        notes: None,
    }
}

/// Statut courant d'une candidature.
fn statut(repo: &SqliteRelanceRepository, candidature_id: Uuid) -> String {
    connexion(&repo.pool)
        .unwrap()
        .query_row(
            "SELECT statut FROM candidatures WHERE id = ?1",
            [candidature_id.to_string()],
            |row| row.get(0),
        )
        .unwrap()
}

mod test_create_ne_touche_pas_au_statut_de_la_candidature;
mod test_create_sur_candidature_inconnue_retourne_une_phrase_lisible;
mod test_la_plage_du_calendrier_inclut_ses_bornes;
mod test_le_poste_et_l_entreprise_sont_aplatis;
mod test_update_identifiant_inconnu_retourne_not_found;
