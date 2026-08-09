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
}
