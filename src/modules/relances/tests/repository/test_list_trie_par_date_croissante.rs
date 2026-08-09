//! Cas de test isolé.

use super::*;

#[test]
fn test_list_trie_par_date_croissante() {
    let repo = repo();
    let cand = candidature(&repo);
    repo.create(&entree(cand, "2026-03-01")).unwrap();
    repo.create(&entree(cand, "2026-01-01")).unwrap();
    let dates: Vec<String> = repo
        .list()
        .unwrap()
        .into_iter()
        .map(|r| r.date_relance)
        .collect();
    assert_eq!(
        dates,
        vec!["2026-01-01".to_string(), "2026-03-01".to_string()]
    );
}
