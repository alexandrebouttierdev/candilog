//! Cas de test isolé.

use super::*;

#[test]
fn la_liste_canonique_est_inseree_dans_l_ordre() {
    let pool = pool();
    let repo = SqliteSecteurRepository::new(pool.clone());
    repo.garantir_referentiel().unwrap();

    let secteurs = repo.lister().unwrap();
    let noms: Vec<&str> = secteurs
        .iter()
        .map(|secteur| secteur.nom.as_str())
        .collect();
    assert_eq!(noms, SECTEURS_CANONIQUES.to_vec());
    assert_eq!(secteurs.len(), SECTEURS_CANONIQUES.len());
}
