//! Cas de test isolé.

use super::*;

#[test]
fn deux_passages_ne_dupliquent_rien() {
    let pool = pool();
    let repo = SqliteSecteurRepository::new(pool.clone());
    repo.garantir_referentiel().unwrap();
    repo.garantir_referentiel().unwrap();

    let secteurs = repo.lister().unwrap();
    assert_eq!(secteurs.len(), SECTEURS_CANONIQUES.len());
}
