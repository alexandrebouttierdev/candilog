//! Cas de test isolé.

use super::*;

/// `docs/DATA.md` exige de « remplacer la base avec possibilité de retour arrière ».
///
/// L'API backup SQLite écrase la cible **en place** : si la restauration échoue en cours de
/// route, la base active reste à moitié écrasée — les données de l'utilisateur sont perdues
/// et le backup n'est pas appliqué pour autant.
///
/// Le fichier source utilisé ici passe la validation (en-tête SQLite, `integrity_check`, les
/// cinq tables attendues) mais fait échouer les migrations rejouées ensuite : sa table
/// `candidatures` n'a pas les colonnes de la 004, et la recopie de la 005 échoue. C'est le cas
/// réaliste d'une base homonyme d'une autre origine.
#[test]
fn import_echoue_sans_detruire_la_base_active() {
    let directory = tempfile::tempdir().unwrap();
    let db_path = directory.path().join("candilog.sqlite");
    let source_path = directory.path().join("etranger.sqlite");

    let pool = open_pool(Some(&db_path)).unwrap();
    run_local_migrations(&pool).unwrap();
    {
        let connection = pool.get().unwrap();
        connection
            .execute_batch(
                "INSERT INTO entreprises (id, nom, created_at, updated_at)
                    VALUES ('e1', 'Acme', '2026-01-01', '2026-01-01');
                 INSERT INTO candidatures
                    (id, entreprise_id, poste, type_contrat, statut, date_envoi, created_at, updated_at)
                    VALUES ('c1', 'e1', 'Dev', 'CDI', 'EN_ATTENTE', '2026-01-01', '2026-01-01', '2026-01-01');",
            )
            .unwrap();
    }

    {
        let connection = rusqlite::Connection::open(&source_path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE candidatures (id TEXT PRIMARY KEY);
                 CREATE TABLE entreprises (id TEXT PRIMARY KEY);
                 CREATE TABLE contacts (id TEXT PRIMARY KEY);
                 CREATE TABLE parametres (id INTEGER PRIMARY KEY);
                 CREATE TABLE profil (id INTEGER PRIMARY KEY);",
            )
            .unwrap();
    }

    let erreur = import(&pool, &db_path, &source_path).unwrap_err();
    assert!(
        erreur.to_string().contains("restaurée"),
        "l'échec doit signaler que la base d'origine a été remise en place : {erreur}"
    );

    // Le retour arrière doit avoir rendu la base active telle qu'elle était.
    let connection = pool.get().unwrap();
    let poste: String = connection
        .query_row(
            "SELECT poste FROM candidatures WHERE id = 'c1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        poste, "Dev",
        "l'import échoué a détruit les données actives"
    );
}
