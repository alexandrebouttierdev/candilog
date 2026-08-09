//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::shared::db::{open_pool, run_local_migrations};

fn repo() -> SqliteMetriquesRepository {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    SqliteMetriquesRepository::new(pool)
}

fn appel(op: OperationLlm, cree_le: &str, succes: bool) -> AppelLlm {
    AppelLlm {
        operation: op,
        provider: "ollama".into(),
        modele: "llama3.2:3b".into(),
        latence_ms: 120,
        succes,
        cree_le: cree_le.into(),
    }
}

mod test_enregistrer_appel_puis_lister_restitue_les_champs;
mod test_enregistrer_score_puis_lister_restitue_les_champs;
mod test_lister_appels_ordonne_du_plus_recent_au_plus_ancien;
mod test_lister_appels_page_borne_et_compte_le_total;
mod test_lister_scores_page_et_resume_restent_globaux;
mod test_reset_appels_vide_le_journal;
mod test_reset_scores_vide_le_journal;
