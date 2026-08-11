use super::*;

#[test]
fn test_relances_a_faire_restent_globales_et_bornees() {
    let repo = repo();
    let company = entreprise(&repo, "ACME");
    let premiere = repo.create(&entree(company, "Développeur Rust")).unwrap();
    let seconde = repo
        .create(&entree(company, "Administrateur Linux"))
        .unwrap();
    repo.update_statut(seconde.id, StatutCandidature::Refus)
        .unwrap();

    let stats = repo.stats().unwrap();
    assert_eq!(stats.to_follow_up, 1);

    let a_relancer = repo.list_to_follow_up("2026-08-05", 5).unwrap();
    assert_eq!(a_relancer.len(), 1);
    assert_eq!(a_relancer[0].id, premiere.id);

    assert!(repo.list_to_follow_up("2026-08-05", 1).unwrap().len() <= 1);
}
