//! Rattachement d'un contact à une candidature : groupes « Recruteurs et managers » et
//! « Réseau » de l'écran Relations.

use super::*;

#[test]
fn un_contact_lie_a_une_candidature_porte_son_activite() {
    let repo = repo();
    let lie = repo.create(&entree("Ménard", None)).unwrap();
    application_liee(&repo, lie.id);

    let activite = repo.get(lie.id).unwrap().activity;
    assert_eq!(activite.applications, 1);
    assert_eq!(activite.last_reference_number, Some(1));
    assert!(activite.last_sent_date.is_some());
}

#[test]
fn le_filtre_de_rattachement_separe_recruteurs_et_reseau() {
    let repo = repo();
    let lie = repo.create(&entree("Ménard", None)).unwrap();
    application_liee(&repo, lie.id);
    repo.create(&entree("Cozic", None)).unwrap();

    let noms = |linked| -> Vec<String> {
        repo.list_page(1, 10, "", None, linked)
            .unwrap()
            .items
            .into_iter()
            .map(|contact| contact.name)
            .collect()
    };
    assert_eq!(noms(Some(true)), vec!["Ménard"]);
    assert_eq!(noms(Some(false)), vec!["Cozic"]);
    assert_eq!(noms(None), vec!["Cozic", "Ménard"]);
}
