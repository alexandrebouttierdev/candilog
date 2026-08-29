//! Cas de test isolé.

use super::*;

/// Le Kanban affiche les quatre colonnes en permanence : si la répartition tenait compte du
/// filtre de statut, sélectionner « Interview » viderait les trois autres en-têtes.
#[test]
fn test_repartition_ignore_le_filtre_de_statut() {
    let (repo, company_id) = context();
    for status in [ApplicationStatus::Pending, ApplicationStatus::Interview] {
        let mut input = entree(company_id, "Développeur", "2026-08-20");
        input.status = status;
        repo.create(&input).unwrap();
    }

    let breakdown = repo
        .breakdown(&ApplicationFilter {
            status: vec![ApplicationStatus::Interview],
            ..ApplicationFilter::default()
        })
        .unwrap();

    assert_eq!(breakdown.pending, 1);
    assert_eq!(breakdown.interview, 1);
}
