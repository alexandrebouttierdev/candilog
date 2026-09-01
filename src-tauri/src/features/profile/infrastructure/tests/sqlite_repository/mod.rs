use super::*;
use crate::core::database::{open_pool, run_local_migrations};

fn repo() -> SqliteProfileRepository {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    SqliteProfileRepository::new(pool)
}

#[test]
fn absence_de_ligne_retourne_un_profil_vide() {
    let (profile, updated_at) = repo().get().unwrap();

    assert_eq!(profile, Profile::default());
    assert!(updated_at.is_none());
}

#[test]
fn json_historique_est_lu_et_restitue_sans_changer_de_schema() {
    let repo = repo();
    connection(&repo.pool)
        .unwrap()
        .execute(
            "INSERT INTO profile (id, data, updated_at) VALUES (1, ?1, '2026-08-01')",
            [r#"{"personal":{"first_name":"Camille","last_name":"Rivet","email":"camille@example.fr"},"experiences":[],"skills":[{"name":"Rust"}],"education":[],"languages":[],"projects":[],"certifications":[]}"#],
        )
        .unwrap();

    let (mut profile, _) = repo.get().unwrap();
    assert_eq!(profile.identity.first_name, "Camille");
    assert_eq!(profile.skills[0].name, "Rust");
    profile.identity.title = Some("Développeuse Rust".into());
    repo.save(&profile).unwrap();

    let json: String = connection(&repo.pool)
        .unwrap()
        .query_row("SELECT data FROM profile WHERE id = 1", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert!(json.contains("\"first_name\""));
    assert!(json.contains("\"headline\":\"Développeuse Rust\""));
    assert!(!json.contains("\"prenom\""));
}

#[test]
fn second_enregistrement_remplace_la_ligne_unique() {
    let repo = repo();
    let mut profile = Profile::default();
    profile.identity.first_name = "Camille".into();
    repo.save(&profile).unwrap();
    profile.identity.first_name = "Noémie".into();

    repo.save(&profile).unwrap();

    let (recharge, updated_at) = repo.get().unwrap();
    assert_eq!(recharge.identity.first_name, "Noémie");
    assert!(updated_at.is_some());
    let count: u64 = connection(&repo.pool)
        .unwrap()
        .query_row("SELECT count(*) FROM profile", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn adresse_postale_est_persistee() {
    let repo = repo();
    let mut profile = Profile::default();
    profile.identity.first_name = "Camille".into();
    profile.identity.address = Some("14 rue Saint-Melaine".into());
    repo.save(&profile).unwrap();

    let (recharge, _) = repo.get().unwrap();
    assert_eq!(
        recharge.identity.address.as_deref(),
        Some("14 rue Saint-Melaine")
    );
}

#[test]
fn json_sans_adresse_reste_lisible() {
    let repo = repo();
    connection(&repo.pool)
        .unwrap()
        .execute(
            "INSERT INTO profile (id, data, updated_at) VALUES (1, ?1, '2026-08-01')",
            [r#"{"personal":{"first_name":"Camille","last_name":"Rivet","email":"camille@example.fr"},"experiences":[],"skills":[],"education":[],"languages":[],"projects":[],"certifications":[]}"#],
        )
        .unwrap();

    let (profile, _) = repo.get().unwrap();
    assert_eq!(profile.identity.address, None);
}
