//! Cas de test isolé.

use super::*;

#[test]
fn test_create_puis_list_expose_le_nom_de_l_entreprise() {
    let repo = repo();
    let ent = entreprise(&repo, "ACME");
    repo.create(&entree(ent, "Dev Rust")).unwrap();
    let liste = repo.list().unwrap();
    assert_eq!(liste.len(), 1);
    assert_eq!(liste[0].poste, "Dev Rust");
    assert_eq!(liste[0].entreprise_nom.as_deref(), Some("ACME"));
}
