//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::core::database::{open_pool, run_local_migrations};
use crate::features::entretiens::domain::TypeEntretien;

/// Dépôt sur base mémoire migrée, avec une candidature déjà créée.
fn contexte() -> (SqliteEntretienRepository, Uuid) {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = connexion(&pool).unwrap();
    let entreprise_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO entreprises (id, nom, created_at, updated_at)
         VALUES (?1, 'Nova Digital', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [entreprise_id.to_string()],
    )
    .unwrap();
    let candidature_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO candidatures (id, entreprise_id, poste, type_contrat, statut, date_envoi,
            created_at, updated_at)
         VALUES (?1, ?2, 'Développeur Frontend', 'CDI', 'EN_ATTENTE', '2026-08-20',
            '2026-08-20T00:00:00Z', '2026-08-20T00:00:00Z')",
        [candidature_id.to_string(), entreprise_id.to_string()],
    )
    .unwrap();
    drop(conn);
    (SqliteEntretienRepository::new(pool), candidature_id)
}

/// Charge utile valide, dont seule la date varie selon les tests.
fn entree(candidature_id: Uuid, date: &str) -> NouvelEntretien {
    NouvelEntretien {
        candidature_id,
        contact_id: None,
        date_entretien: date.into(),
        type_entretien: TypeEntretien::Visio,
        lieu: Some("https://meet.example/abc".into()),
        notes: None,
        compte_rendu: None,
    }
}

/// Statut courant d'une candidature.
fn statut(repo: &SqliteEntretienRepository, candidature_id: Uuid) -> String {
    connexion(&repo.pool)
        .unwrap()
        .query_row(
            "SELECT statut FROM candidatures WHERE id = ?1",
            [candidature_id.to_string()],
            |row| row.get(0),
        )
        .unwrap()
}

/// Nombre d'étapes enregistrées dans l'historique de statut.
fn etapes(repo: &SqliteEntretienRepository, candidature_id: Uuid) -> i64 {
    connexion(&repo.pool)
        .unwrap()
        .query_row(
            "SELECT count(*) FROM statut_history WHERE candidature_id = ?1 AND statut = 'ENTRETIEN'",
            [candidature_id.to_string()],
            |row| row.get(0),
        )
        .unwrap()
}

mod test_enregistrer_deux_fois_n_historise_qu_une_etape;
mod test_enregistrer_fait_avancer_la_candidature;
mod test_enregistrer_sur_candidature_inconnue_retourne_une_phrase_lisible;
mod test_enregistrer_sur_identifiant_inconnu_retourne_not_found;
mod test_la_plage_du_calendrier_inclut_ses_bornes;
mod test_la_suppression_conserve_le_statut_de_la_candidature;
mod test_le_poste_et_l_entreprise_sont_aplatis;
