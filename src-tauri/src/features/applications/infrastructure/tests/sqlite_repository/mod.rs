//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::core::database::{open_pool, run_local_migrations};
use crate::features::applications::domain::ContractType;

/// Dépôt sur base mémoire migrée, avec une entreprise déjà créée.
fn context() -> (SqliteApplicationRepository, Uuid) {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let company_id = Uuid::new_v4();
    connection(&pool)
        .unwrap()
        .execute(
            "INSERT INTO companies (id, name, city, created_at, updated_at)
             VALUES (?1, 'Nova Digital', 'Rennes', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [company_id.to_string()],
        )
        .unwrap();
    (SqliteApplicationRepository::new(pool), company_id)
}

/// Payload utile valide, dont seuls le poste et la date varient selon les tests.
fn entree(company_id: Uuid, job_title: &str, sent_date: &str) -> NewApplication {
    NewApplication {
        job_title: job_title.into(),
        company_id,
        contract_type: ContractType::Cdi,
        status: ApplicationStatus::Pending,
        sent_date: sent_date.into(),
        job_url: None,
        notes: None,
    }
}

/// Étapes enregistrées dans l'historique de statut d'une candidature, dans l'ordre.
fn history(repo: &SqliteApplicationRepository, id: Uuid) -> Vec<String> {
    let conn = connection(&repo.pool).unwrap();
    let mut query = conn
        .prepare(
            "SELECT status FROM status_history WHERE application_id = ?1 ORDER BY changed_at ASC",
        )
        .unwrap();
    let rows = query
        .query_map([id.to_string()], |row| row.get(0))
        .unwrap();
    rows.map(Result::unwrap).collect()
}

mod test_changer_statut_pour_la_meme_valeur_n_ajoute_pas_d_etape;
mod test_create_ouvre_l_historique_de_statut;
mod test_create_sur_entreprise_inconnue_retourne_une_phrase_lisible;
mod test_delete_efface_l_historique_en_cascade;
mod test_get_identifiant_inconnu_retourne_not_found;
mod test_le_filtre_de_periode_borne_les_deux_extremites;
mod test_le_filtre_par_identifiants_restreint_l_export;
mod test_le_filtre_par_statuts_retient_toutes_les_valeurs_cochees;
mod test_le_tri_par_entreprise_ignore_la_casse;
mod test_pagination_applique_les_filtres_avant_la_limite;
mod test_repartition_compte_les_quatre_statuts;
mod test_repartition_ignore_le_filtre_de_statut;
mod test_update_n_historise_que_les_changements_reels;
