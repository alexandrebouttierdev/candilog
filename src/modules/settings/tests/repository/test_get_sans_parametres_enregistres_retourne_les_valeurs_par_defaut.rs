//! Cas de test isolé.

use super::*;

#[test]
fn test_get_sans_parametres_enregistres_retourne_les_valeurs_par_defaut() {
    let repo = repo();
    assert_eq!(repo.get().unwrap(), AppSettings::default());
}
