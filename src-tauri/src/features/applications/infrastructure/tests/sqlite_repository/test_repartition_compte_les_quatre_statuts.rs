//! Cas de test isolé.

use super::*;

#[test]
fn test_repartition_compte_les_quatre_statuts() {
    let (repo, company_id) = context();
    for status in [
        ApplicationStatus::Pending,
        ApplicationStatus::Pending,
        ApplicationStatus::Interview,
        ApplicationStatus::Rejected,
    ] {
        let mut input = entree(company_id, "Développeur", "2026-08-20");
        input.status = status;
        repo.create(&input).unwrap();
    }

    let breakdown = repo.breakdown(&ApplicationFilter::default()).unwrap();

    assert_eq!(breakdown.pending, 2);
    assert_eq!(breakdown.followed_up, 0);
    assert_eq!(breakdown.interview, 1);
    assert_eq!(breakdown.rejected, 1);
}
