//! Cas de test isolé.

use super::*;

#[test]
fn test_list_trie_par_nom_puis_prenom() {
    let repo = repo();
    repo.create(&entree("Zoe", None)).unwrap();
    repo.create(&entree("Adam", None)).unwrap();
    let noms: Vec<String> = repo.list().unwrap().into_iter().map(|c| c.nom).collect();
    assert_eq!(noms, vec!["Adam".to_string(), "Zoe".to_string()]);
}
