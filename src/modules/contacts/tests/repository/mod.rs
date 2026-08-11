//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::shared::db::{open_pool, run_local_migrations};

fn repo() -> SqliteContactRepository {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    SqliteContactRepository::new(pool)
}

fn entree(nom: &str, entreprise_id: Option<uuid::Uuid>) -> NouveauContact {
    NouveauContact {
        entreprise_id,
        prenom: "Alex".into(),
        nom: nom.into(),
        poste: Some("CTO".into()),
        email: Some("alex@example.com".into()),
        telephone: None,
        linkedin: None,
        notes: None,
    }
}

fn entreprise(repo: &SqliteContactRepository) -> uuid::Uuid {
    let id = uuid::Uuid::new_v4();
    let conn = crate::shared::sqlite::connexion(&repo.pool).unwrap();
    conn.execute(
        "INSERT INTO entreprises (id, nom, created_at, updated_at)
             VALUES (?1, 'ACME', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [id.to_string()],
    )
    .unwrap();
    id
}

/// Insère une candidature rattachée au contact, en `SQL` direct : le module `contacts`
/// n'importe pas le module `candidatures`.
fn candidature_liee(repo: &SqliteContactRepository, contact_id: uuid::Uuid) -> uuid::Uuid {
    let entreprise_id = entreprise(repo);
    let id = uuid::Uuid::new_v4();
    let conn = crate::shared::sqlite::connexion(&repo.pool).unwrap();
    conn.execute(
            "INSERT INTO candidatures (id, entreprise_id, contact_id, poste, date_envoi, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'Dev', '2026-01-01', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            rusqlite::params![id.to_string(), entreprise_id.to_string(), contact_id.to_string()],
        )
        .unwrap();
    id
}

mod test_create_entreprise_inconnue_retourne_une_phrase_lisible;
mod test_create_puis_list_restitue_le_contact_et_son_entreprise;
mod test_delete_contact_lie_a_un_entretien_seul_est_refuse;
mod test_delete_contact_lie_a_une_candidature_est_refuse;
mod test_delete_supprime_le_contact;
mod test_list_trie_par_nom_puis_prenom;
mod test_pagination_applique_la_recherche_avant_la_limite;
mod test_suppression_entreprise_detache_le_contact_sans_le_supprimer;
mod test_update_entreprise_inconnue_retourne_une_phrase_lisible;
mod test_update_identifiant_inconnu_retourne_not_found;
mod test_update_modifie_les_champs;
