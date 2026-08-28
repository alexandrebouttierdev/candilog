//! Cas de test isolé.

use super::*;

#[test]
fn test_update_entreprise_inconnue_retourne_une_phrase_lisible() {
    let repo = repo();
    let cree = repo.create(&entree("Bouttier", None)).unwrap();
    let resultat = repo.update(cree.id, &entree("Bouttier", Some(uuid::Uuid::new_v4())));
    match resultat {
        Err(AppError::Validation(message)) => assert_eq!(
            message, "L'entreprise liée à ce contact est introuvable",
            "message rendu à l'utilisateur"
        ),
        autre => panic!("attendu Validation, obtenu {autre:?}"),
    }
}
