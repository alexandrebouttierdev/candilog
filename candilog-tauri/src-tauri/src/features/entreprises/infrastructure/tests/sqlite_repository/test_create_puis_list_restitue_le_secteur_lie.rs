//! Cas de test isolé.

use super::*;
use crate::features::secteurs::domain::SecteurRepository;
use crate::features::secteurs::infrastructure::SqliteSecteurRepository;

#[test]
fn create_puis_list_restitue_le_secteur_lie() {
    let pool = open_pool(None).unwrap();
    run_local_migrations(&pool).unwrap();
    let secteurs = SqliteSecteurRepository::new(pool.clone());
    secteurs.garantir_referentiel().unwrap();
    let repo = SqliteEntrepriseRepository::new(pool);
    let reference = secteurs.lister().unwrap().remove(0);

    let creee = repo
        .create(&NouvelleEntreprise {
            nom: "Agrial".into(),
            secteur_id: Some(reference.id),
            secteur: Some(reference.nom.clone()),
            type_: None,
            site_web: None,
            ville: None,
            adresse: None,
            notes: None,
        })
        .unwrap();
    assert_eq!(creee.secteur_id, Some(reference.id));
    assert_eq!(creee.secteur.as_deref(), Some(reference.nom.as_str()));

    let liste = repo.list().unwrap();
    assert_eq!(liste.len(), 1);
    assert_eq!(liste[0].secteur_id, Some(reference.id));
    assert_eq!(liste[0].secteur.as_deref(), Some(reference.nom.as_str()));
}
