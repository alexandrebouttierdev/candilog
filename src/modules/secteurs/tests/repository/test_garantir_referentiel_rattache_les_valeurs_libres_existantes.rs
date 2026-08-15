//! Cas de test isolé.

use super::*;
use crate::modules::entreprises::model::NouvelleEntreprise;
use crate::modules::entreprises::repository::{EntrepriseRepository, SqliteEntrepriseRepository};

#[test]
fn les_valeurs_libres_existantes_sont_rattachees() {
    let pool = pool();
    let entreprises = SqliteEntrepriseRepository::new(pool.clone());
    let secteurs = SqliteSecteurRepository::new(pool.clone());
    // Valeur libre déjà couverte par la liste canonique.
    entreprises
        .create(&NouvelleEntreprise {
            nom: "Atelier d'art".into(),
            secteur_id: None,
            secteur: Some("Arts / Artisanat d'art".into()),
            type_: None,
            site_web: None,
            ville: None,
            adresse: None,
            notes: None,
        })
        .unwrap();
    // Valeur libre hors liste canonique : elle doit devenir une ligne du référentiel.
    entreprises
        .create(&NouvelleEntreprise {
            nom: "Ferme Bio".into(),
            secteur_id: None,
            secteur: Some("agriculture bio".into()),
            type_: None,
            site_web: None,
            ville: None,
            adresse: None,
            notes: None,
        })
        .unwrap();

    secteurs.garantir_referentiel().unwrap();

    let repertoire = entreprises.list().unwrap();
    let lignes_du_referentiel = secteurs.lister().unwrap();
    assert_eq!(lignes_du_referentiel.len(), SECTEURS_CANONIQUES.len() + 1);

    let atelier = repertoire
        .iter()
        .find(|item| item.nom == "Atelier d'art")
        .unwrap();
    let arts = lignes_du_referentiel
        .iter()
        .find(|item| item.nom == "Arts / Artisanat d'art")
        .unwrap();
    assert_eq!(atelier.secteur_id, Some(arts.id));

    let ferme = repertoire
        .iter()
        .find(|item| item.nom == "Ferme Bio")
        .unwrap();
    assert!(ferme.secteur_id.is_some());
    let bio = lignes_du_referentiel
        .iter()
        .find(|item| item.id == ferme.secteur_id.unwrap())
        .unwrap();
    assert_eq!(bio.nom, "agriculture bio");
    assert!(!SECTEURS_CANONIQUES.contains(&bio.nom.as_str()));
}
