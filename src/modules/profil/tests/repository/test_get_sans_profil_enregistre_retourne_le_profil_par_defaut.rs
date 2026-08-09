//! Cas de test isolé.

use super::*;

#[test]
fn test_get_sans_profil_enregistre_retourne_le_profil_par_defaut() {
    let repo = repo();
    assert_eq!(repo.get().unwrap(), Profile::default());
}
