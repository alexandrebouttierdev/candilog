//! Vérification des agrégats `SQLite` sur une base mémoire migrée.

use super::*;
use crate::core::database::{open_pool, run_local_migrations};
use uuid::Uuid;

fn contexte() -> (SqliteAnalysesRepository, Uuid) {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let entreprise = Uuid::new_v4();
    connexion(&pool)
        .unwrap()
        .execute(
            "INSERT INTO entreprises (id, nom, ville, created_at, updated_at)
             VALUES (?1, 'Nova Digital', 'Rennes', '2026-01-01', '2026-01-01')",
            [entreprise.to_string()],
        )
        .unwrap();
    (SqliteAnalysesRepository::new(pool), entreprise)
}

fn candidature(
    repo: &SqliteAnalysesRepository,
    entreprise: Uuid,
    statut: &str,
    date: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    connexion(&repo.pool)
        .unwrap()
        .execute(
            "INSERT INTO candidatures (
                id, entreprise_id, poste, type_contrat, statut, date_envoi, created_at, updated_at
             ) VALUES (?1, ?2, 'Développeur Rust', 'CDI', ?3, ?4, ?4, ?4)",
            rusqlite::params![id.to_string(), entreprise.to_string(), statut, date],
        )
        .unwrap();
    id
}

#[test]
fn indicateurs_conservent_les_etapes_atteintes_apres_un_refus() {
    let (repo, entreprise) = contexte();
    let refusee = candidature(&repo, entreprise, "REFUS", "2026-08-10");
    candidature(&repo, entreprise, "EN_ATTENTE", "2026-08-12");
    candidature(&repo, entreprise, "RELANCEE", "2026-06-01");
    connexion(&repo.pool)
        .unwrap()
        .execute(
            "INSERT INTO statut_history (id, candidature_id, statut, changed_at)
             VALUES (?1, ?2, 'ENTRETIEN', '2026-08-18')",
            rusqlite::params![Uuid::new_v4().to_string(), refusee.to_string()],
        )
        .unwrap();

    let indicateurs = repo.indicateurs(Some("2026-08-01")).unwrap();

    assert_eq!(indicateurs.candidatures, 2);
    assert_eq!(indicateurs.entretiens, 1);
    assert_eq!(indicateurs.reponses, 1);
    assert_eq!(indicateurs.refus, 1);
    assert_eq!(indicateurs.en_attente, 1);
    assert_eq!(indicateurs.taux_reponse, 50);
}

#[test]
fn activite_retourne_toutes_les_semaines_meme_vides() {
    let (repo, entreprise) = contexte();
    let aujourd_hui = chrono::Local::now().date_naive();
    let cette_semaine = (aujourd_hui - chrono::Duration::days(2))
        .format("%Y-%m-%d")
        .to_string();
    let semaine_precedente = (aujourd_hui - chrono::Duration::days(9))
        .format("%Y-%m-%d")
        .to_string();
    candidature(&repo, entreprise, "EN_ATTENTE", &cette_semaine);
    candidature(&repo, entreprise, "EN_ATTENTE", &semaine_precedente);

    let activite = repo.activite_hebdomadaire(4).unwrap();

    assert_eq!(activite.len(), 4);
    assert_eq!(activite[2].nombre, 1);
    assert_eq!(activite[3].nombre, 1);
}

#[test]
fn echeances_ne_retiennent_que_le_futur_et_restent_ordonnees() {
    let (repo, entreprise) = contexte();
    let candidature = candidature(&repo, entreprise, "EN_ATTENTE", "2026-08-01");
    let conn = connexion(&repo.pool).unwrap();
    for (date, type_relance) in [("2026-08-20", "Email"), ("2026-09-02", "Téléphone")] {
        conn.execute(
            "INSERT INTO relances (id, candidature_id, date_relance, type, created_at)
             VALUES (?1, ?2, ?3, ?4, ?3)",
            rusqlite::params![
                Uuid::new_v4().to_string(),
                candidature.to_string(),
                date,
                type_relance
            ],
        )
        .unwrap();
    }
    conn.execute(
        "INSERT INTO entretiens (
            id, candidature_id, date_entretien, type, created_at, updated_at
         ) VALUES (?1, ?2, '2026-09-01T14:00:00+02:00', 'Visio', '2026-08-01', '2026-08-01')",
        rusqlite::params![Uuid::new_v4().to_string(), candidature.to_string()],
    )
    .unwrap();

    let echeances = repo.echeances("2026-08-28", 5).unwrap();

    assert_eq!(echeances.len(), 2);
    assert_eq!(echeances[0].genre, "entretien");
    assert_eq!(echeances[1].genre, "relance");
}

#[test]
fn candidatures_a_relancer_respectent_age_statut_et_limite() {
    let (repo, entreprise) = contexte();
    candidature(&repo, entreprise, "EN_ATTENTE", "2026-08-10");
    candidature(&repo, entreprise, "EN_ATTENTE", "2026-08-25");
    candidature(&repo, entreprise, "REFUS", "2026-08-01");

    let items = repo.a_relancer("2026-08-28", 7, 1).unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].jours, 18);
}

#[test]
fn recentes_restituent_les_jointures_et_les_enums_du_domaine() {
    let (repo, entreprise) = contexte();
    candidature(&repo, entreprise, "ENTRETIEN", "2026-08-20");

    let items = repo.recentes(3).unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].entreprise_nom.as_deref(), Some("Nova Digital"));
    assert_eq!(items[0].statut, StatutCandidature::Entretien);
    assert_eq!(items[0].type_contrat, TypeContrat::Cdi);
}
