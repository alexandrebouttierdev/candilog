//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::core::database::{open_pool, run_local_migrations};
use crate::features::applications::domain::{ApplicationType, WeeklyWorkSchedule};

/// Dépôt sur base mémoire migrée, avec une entreprise déjà créée.
///
/// L'entreprise porte ville, adresse et type : c'est le socle dont les candidatures
/// héritent tant qu'elles ne surchargent rien.
fn context() -> (SqliteApplicationRepository, Uuid) {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let company_id = Uuid::new_v4();
    connection(&pool)
        .unwrap()
        .execute(
            "INSERT INTO companies (id, name, city, address, company_type_id, company_size,
                created_at, updated_at)
             VALUES (?1, 'Nova Digital', 'Rennes', '12 rue des Lilas', 'IT_SERVICES_COMPANY',
                'PME', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [company_id.to_string()],
        )
        .unwrap();
    (SqliteApplicationRepository::new(pool), company_id)
}

/// Crée une entreprise supplémentaire et renvoie son identifiant.
fn autre_entreprise(
    repo: &SqliteApplicationRepository,
    name: &str,
    city: &str,
    company_type_id: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    connection(&repo.pool)
        .unwrap()
        .execute(
            "INSERT INTO companies (id, name, city, company_type_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            rusqlite::params![id.to_string(), name, city, company_type_id],
        )
        .unwrap();
    id
}

/// Payload utile valide, dont seuls le poste et la date varient selon les tests.
///
/// Aucune surcharge : la candidature hérite de tout ce que porte son entreprise.
fn entree(company_id: Uuid, job_title: &str, sent_date: &str) -> NewApplication {
    NewApplication {
        job_title: job_title.into(),
        company_id,
        contact_id: None,
        application_type: ApplicationType::JobOffer,
        contract_type_code: "CDI".into(),
        weekly_work_schedule: WeeklyWorkSchedule::Unspecified,
        weekly_hours: None,
        professional_domain_id: None,
        city: None,
        address: None,
        company_type_id: None,
        status: ApplicationStatus::Pending,
        sent_date: sent_date.into(),
        job_url: Some("https://example.org/offre".into()),
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
    let rows = query.query_map([id.to_string()], |row| row.get(0)).unwrap();
    rows.map(Result::unwrap).collect()
}

mod test_changer_statut_pour_la_meme_valeur_n_ajoute_pas_d_etape;
mod test_create_ouvre_l_historique_de_statut;
mod test_create_sur_entreprise_inconnue_retourne_une_phrase_lisible;
mod test_delete_efface_l_historique_en_cascade;
mod test_delete_identifiant_inconnu_retourne_not_found;
mod test_get_identifiant_inconnu_retourne_not_found;
mod test_la_duree_hebdomadaire_est_persistee_et_relue;
mod test_la_recherche_ignore_les_accents_et_la_casse;
mod test_le_changement_d_entreprise_actualise_l_heritage;
mod test_le_filtre_de_periode_borne_les_deux_extremites;
mod test_le_filtre_par_heures_hebdomadaires_borne_les_deux_cotes;
mod test_le_filtre_par_identifiants_restreint_l_export;
mod test_le_filtre_par_statuts_retient_toutes_les_valeurs_cochees;
mod test_le_filtre_par_ville_effective_retient_les_valeurs_heritees;
mod test_le_tri_par_entreprise_ignore_la_casse;
mod test_les_libelles_des_referentiels_sont_resolus_par_jointure;
mod test_les_surcharges_priment_et_leur_retrait_restitue_l_heritage;
mod test_pagination_applique_les_filtres_avant_la_limite;
mod test_repartition_compte_les_quatre_statuts;
mod test_repartition_ignore_le_filtre_de_statut;
mod test_un_contrat_inconnu_est_refuse;
mod test_update_n_historise_que_les_changements_reels;
