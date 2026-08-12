//! Cas de test isolé.

use super::*;

#[test]
fn test_get_parametres_illisibles_retourne_les_valeurs_par_defaut() {
    // Un JSON corrompu ne doit pas empêcher l'application de démarrer.
    let repo = repo();
    {
        let conn = crate::shared::sqlite::connexion(&repo.pool).unwrap();
        conn.execute(
            "INSERT INTO parametres (id, data, updated_at) VALUES (1, 'pas du json', ?1)",
            [crate::shared::sqlite::maintenant_iso()],
        )
        .unwrap();
    }
    assert_eq!(repo.get().unwrap(), AppSettings::default());
    let conn = crate::shared::sqlite::connexion(&repo.pool).unwrap();
    let backup: String = conn
        .query_row(
            "SELECT valeur FROM app_kv WHERE cle = 'parametres_corrompus'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(backup, "pas du json");
}
