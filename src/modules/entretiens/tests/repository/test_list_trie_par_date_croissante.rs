//! Cas de test isolé.

use super::*;

#[test]
fn test_list_trie_par_date_croissante() {
    let repo = repo();
    let cand = candidature(&repo);
    repo.create(&entree(cand, "2026-05-01T10:00:00Z")).unwrap();
    repo.create(&entree(cand, "2026-04-01T10:00:00Z")).unwrap();
    let dates: Vec<String> = repo
        .list()
        .unwrap()
        .into_iter()
        .map(|e| e.date_entretien)
        .collect();
    assert_eq!(dates[0], "2026-04-01T10:00:00Z");
}
