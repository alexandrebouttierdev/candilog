//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::modules::candidatures::model::TypeContrat;
use crate::shared::db::{open_pool, run_local_migrations};

fn repo() -> SqliteCandidatureRepository {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    SqliteCandidatureRepository::new(pool)
}

fn entreprise(repo: &SqliteCandidatureRepository, nom: &str) -> uuid::Uuid {
    let id = uuid::Uuid::new_v4();
    let conn = crate::shared::sqlite::connexion(&repo.pool).unwrap();
    conn.execute(
        "INSERT INTO entreprises (id, nom, created_at, updated_at)
             VALUES (?1, ?2, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        rusqlite::params![id.to_string(), nom],
    )
    .unwrap();
    id
}

fn entree(entreprise_id: uuid::Uuid, poste: &str) -> NouvelleCandidature {
    NouvelleCandidature {
        poste: poste.into(),
        entreprise_id,
        type_contrat: TypeContrat::Cdi,
        statut: StatutCandidature::EnAttente,
        date_envoi: "2026-01-15".into(),
        lien_offre: None,
        notes: None,
    }
}

mod test_create_conserve_type_contrat_et_statut;
mod test_create_entreprise_inconnue_retourne_validation;
mod test_create_puis_list_expose_le_nom_de_l_entreprise;
mod test_delete_supprime_la_candidature_et_ses_relances;
mod test_list_trie_les_plus_recentes_d_abord;
mod test_pagination_recherche_et_agregats_restent_globaux;
mod test_relances_a_faire_restent_globales_et_bornees;
mod test_update_entreprise_inconnue_retourne_validation;
mod test_update_identifiant_inconnu_retourne_not_found;
mod test_update_modifie_les_champs_editables;
mod test_update_preserve_le_contact_lie;
mod test_update_statut_identifiant_inconnu_retourne_not_found;
mod test_update_statut_modifie_uniquement_le_statut;
