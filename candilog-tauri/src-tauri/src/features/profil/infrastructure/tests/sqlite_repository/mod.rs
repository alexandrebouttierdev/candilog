use super::*;
use crate::core::database::{open_pool, run_local_migrations};

fn depot() -> SqliteProfilRepository {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    SqliteProfilRepository::new(pool)
}

#[test]
fn absence_de_ligne_retourne_un_profil_vide() {
    let (profil, updated_at) = depot().obtenir().unwrap();

    assert_eq!(profil, Profil::default());
    assert!(updated_at.is_none());
}

#[test]
fn json_historique_est_lu_et_restitue_sans_changer_de_schema() {
    let repo = depot();
    connexion(&repo.pool)
        .unwrap()
        .execute(
            "INSERT INTO profil (id, data, updated_at) VALUES (1, ?1, '2026-08-01')",
            [r#"{"personal":{"first_name":"Camille","last_name":"Rivet","email":"camille@example.fr"},"experiences":[],"skills":[{"name":"Rust"}],"education":[],"languages":[],"projects":[],"certifications":[]}"#],
        )
        .unwrap();

    let (mut profil, _) = repo.obtenir().unwrap();
    assert_eq!(profil.identite.prenom, "Camille");
    assert_eq!(profil.competences[0].nom, "Rust");
    profil.identite.titre = Some("Développeuse Rust".into());
    repo.enregistrer(&profil).unwrap();

    let json: String = connexion(&repo.pool)
        .unwrap()
        .query_row("SELECT data FROM profil WHERE id = 1", [], |row| row.get(0))
        .unwrap();
    assert!(json.contains("\"first_name\""));
    assert!(json.contains("\"headline\":\"Développeuse Rust\""));
    assert!(!json.contains("\"prenom\""));
}

#[test]
fn second_enregistrement_remplace_la_ligne_unique() {
    let repo = depot();
    let mut profil = Profil::default();
    profil.identite.prenom = "Camille".into();
    repo.enregistrer(&profil).unwrap();
    profil.identite.prenom = "Noémie".into();

    repo.enregistrer(&profil).unwrap();

    let (recharge, updated_at) = repo.obtenir().unwrap();
    assert_eq!(recharge.identite.prenom, "Noémie");
    assert!(updated_at.is_some());
    let count: u64 = connexion(&repo.pool)
        .unwrap()
        .query_row("SELECT count(*) FROM profil", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);
}
