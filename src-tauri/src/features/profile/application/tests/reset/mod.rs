//! Périmètre réel de `profile_reset`, sur une base SQLite complète.
//!
//! Le test du service utilise un dépôt en mémoire, qui ne connaît que le profil : il ne
//! peut donc rien dire de ce que la réinitialisation épargne. Ici la base porte les autres
//! tables, et chacune est recomptée après l'opération.

use crate::core::database::helpers::connection;
use crate::core::database::{open_pool, run_local_migrations, SqlitePool};
use crate::features::profile::application::ProfileService;
use crate::features::profile::domain::{Identity, Profile, Skill};
use crate::features::profile::infrastructure::SqliteProfileRepository;

/// Base migrée, garnie d'une donnée par table que le reset ne doit pas toucher.
fn base_garnie() -> SqlitePool {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = connection(&pool).unwrap();
    conn.execute_batch(
        "INSERT INTO companies (id, name, company_size, created_at, updated_at)
             VALUES ('c1', 'Nova Digital', 'PME', '2026-08-01', '2026-08-01');
         INSERT INTO contacts (id, company_id, first_name, name, created_at, updated_at)
             VALUES ('ct1', 'c1', 'Camille', 'Rivet', '2026-08-01', '2026-08-01');
         INSERT INTO applications
             (id, company_id, job_title, contract_type_code, status, sent_date, created_at, updated_at)
             VALUES ('a1', 'c1', 'Développeur', 'CDI', 'EN_ATTENTE', '2026-08-01', '2026-08-01', '2026-08-01');
         INSERT INTO status_history (id, application_id, status, changed_at)
             VALUES ('h1', 'a1', 'EN_ATTENTE', '2026-08-01');
         INSERT INTO follow_ups (id, application_id, follow_up_date, type, created_at)
             VALUES ('f1', 'a1', '2026-08-10', 'Email', '2026-08-01');
         INSERT INTO interviews (id, application_id, interview_date, created_at, updated_at)
             VALUES ('i1', 'a1', '2026-08-12T10:00:00Z', '2026-08-01', '2026-08-01');
         INSERT INTO resume_versions (id, name, content, created_at)
             VALUES ('r1', 'CV Rust', '{}', '2026-08-01');
         INSERT INTO cover_letters (id, name, content, created_at)
             VALUES ('l1', 'Lettre Nova', 'Bonjour', '2026-08-01');
         INSERT INTO settings (id, data, updated_at) VALUES (1, '{\"theme\":\"dark\"}', '2026-08-01');
         INSERT INTO app_kv (kv_key, kv_value) VALUES ('temoin', 'intact');",
    )
    .unwrap();
    pool
}

fn compte(pool: &SqlitePool, table: &str) -> i64 {
    connection(pool)
        .unwrap()
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
}

#[test]
fn reinitialiser_le_profil_ne_touche_a_aucune_autre_table() {
    let pool = base_garnie();
    let photos = tempfile::tempdir().unwrap();
    let service = ProfileService::new(
        SqliteProfileRepository::new(pool.clone()),
        photos.path().to_path_buf(),
    );
    service
        .save(&Profile {
            identity: Identity {
                first_name: "Camille".into(),
                name: "Rivet".into(),
                email: "camille@example.fr".into(),
                ..Identity::default()
            },
            skills: vec![Skill {
                name: "Rust".into(),
            }],
            ..Profile::default()
        })
        .unwrap();

    let payload = service.reset().unwrap();

    assert_eq!(payload.profile, Profile::default());
    assert_eq!(service.load().unwrap().profile, Profile::default());

    // Tout le reste des données utilisateur est intact, à la ligne près.
    for table in [
        "companies",
        "contacts",
        "applications",
        "status_history",
        "follow_ups",
        "interviews",
        "resume_versions",
        "cover_letters",
        "settings",
        "app_kv",
    ] {
        assert_eq!(compte(&pool, table), 1, "la table {table} a été modifiée");
    }

    // Les référentiels métier semés par le schéma sont eux aussi préservés.
    assert!(compte(&pool, "sectors") > 0);
    assert!(compte(&pool, "contract_types") > 0);

    // Et les réglages n'ont pas perdu leur contenu.
    let theme: String = connection(&pool)
        .unwrap()
        .query_row("SELECT data FROM settings WHERE id = 1", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert!(theme.contains("dark"));
}
