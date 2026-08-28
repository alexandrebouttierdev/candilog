//! Cas de test isolé.

use super::*;

#[test]
fn test_list_trie_par_nom_croissant() {
    let repo = repo();
    repo.create(&entree("Zeta")).unwrap();
    repo.create(&entree("Alpha")).unwrap();
    let names: Vec<String> = repo.list().unwrap().into_iter().map(|e| e.name).collect();
    assert_eq!(names, vec!["Alpha".to_string(), "Zeta".to_string()]);
}
