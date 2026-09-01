//! Cas de test isolé.

use super::*;

fn colonnes(conn: &rusqlite::Connection, table: &str) -> Vec<String> {
    let mut query = conn
        .prepare(&format!("SELECT name FROM pragma_table_info('{table}')"))
        .unwrap();
    let rows = query.query_map([], |row| row.get(0)).unwrap();
    rows.map(Result::unwrap).collect()
}

#[test]
fn une_base_neuve_expose_les_colonnes_de_mise_en_page() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    let cover_letters = colonnes(&conn, "cover_letters");
    for attendue in ["recipient", "recipient_address", "job_reference"] {
        assert!(
            cover_letters.contains(&attendue.to_owned()),
            "cover_letters.{attendue} absente"
        );
    }
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, LATEST_SCHEMA_VERSION);
}

#[test]
fn une_base_v1_recoit_les_colonnes_sans_perdre_les_lettres() {
    let pool = open_pool(None).unwrap();
    {
        let conn = pool.get().unwrap();
        conn.execute_batch(
            "CREATE TABLE cover_letters (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                company TEXT,
                job_title TEXT,
                tone TEXT NOT NULL DEFAULT 'formal',
                length TEXT NOT NULL DEFAULT 'medium',
                content TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            INSERT INTO cover_letters (id, name, tone, length, content, created_at)
            VALUES ('lettre-1', 'Lettre Nova', 'formal', 'medium', 'Madame,', '2026-01-01');",
        )
        .unwrap();
        conn.pragma_update(None, "user_version", 1).unwrap();
    }

    run_local_migrations(&pool).unwrap();
    let conn = pool.get().unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 2);
    let cover_letters = colonnes(&conn, "cover_letters");
    for attendue in ["recipient", "recipient_address", "job_reference"] {
        assert!(
            cover_letters.contains(&attendue.to_owned()),
            "cover_letters.{attendue} absente après migration v1"
        );
    }
    let name: String = conn
        .query_row(
            "SELECT name FROM cover_letters WHERE id = 'lettre-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(name, "Lettre Nova");
}
