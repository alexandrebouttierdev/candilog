//! Cas de test isolé.

use super::*;

/// Le libellé du secteur n'est pas stocké : il est résolu par jointure, ce qui évite deux
/// sources de vérité que rien ne garderait d'accord.
#[test]
fn create_puis_list_restitue_le_secteur_lie() {
    let repo = repo();
    let sector_id = uuid::Uuid::parse_str(SECTEUR_INFORMATIQUE).unwrap();
    let mut entree = entree("Agrial");
    entree.sector_id = Some(sector_id);

    let creee = repo.create(&entree).unwrap();
    assert_eq!(creee.sector_id, Some(sector_id));
    assert_eq!(
        creee.sector_name.as_deref(),
        Some("Informatique / Télécommunication")
    );

    let list = repo.list().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].sector_id, Some(sector_id));
    assert_eq!(
        list[0].sector_name.as_deref(),
        Some("Informatique / Télécommunication")
    );
}

#[test]
fn un_secteur_hors_referentiel_est_refuse() {
    let repo = repo();
    let mut entree = entree("Agrial");
    entree.sector_id = Some(uuid::Uuid::new_v4());

    assert!(matches!(repo.create(&entree), Err(AppError::Validation(_))));
}
