//! Cas de test isolé.

use super::*;

#[test]
fn test_list_trie_les_plus_recentes_d_abord() {
    let repo = repo();
    let ent = entreprise(&repo, "ACME");
    let mut ancienne = entree(ent, "Ancienne");
    ancienne.date_envoi = "2026-01-01".into();
    let mut recente = entree(ent, "Récente");
    recente.date_envoi = "2026-06-01".into();
    repo.create(&ancienne).unwrap();
    repo.create(&recente).unwrap();
    let postes: Vec<String> = repo.list().unwrap().into_iter().map(|c| c.poste).collect();
    assert_eq!(postes, vec!["Récente".to_string(), "Ancienne".to_string()]);
}
