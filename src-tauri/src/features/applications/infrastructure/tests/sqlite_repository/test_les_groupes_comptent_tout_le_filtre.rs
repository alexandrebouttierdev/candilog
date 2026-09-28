//! Cas de test isolé.

use super::*;

#[test]
fn test_les_groupes_comptent_tout_le_filtre() {
    let (repo, nova) = context();
    let astrea = autre_entreprise(&repo, "Groupe Astréa", "Nantes", "FINAL_CLIENT");
    for (company, contract, status) in [
        (nova, "CDI", ApplicationStatus::Pending),
        (nova, "CDD", ApplicationStatus::Pending),
        (astrea, "CDI", ApplicationStatus::Pending),
        (astrea, "CDI", ApplicationStatus::Interview),
        (astrea, "CDI", ApplicationStatus::Rejected),
    ] {
        let mut input = entree(company, "Technicien", "2026-08-20");
        input.contract_type_code = contract.into();
        input.status = status;
        repo.create(&input).unwrap();
    }

    let par_entreprise = repo
        .groups(&ApplicationFilter::default(), ApplicationGrouping::Company)
        .unwrap();
    assert_eq!(
        par_entreprise
            .iter()
            .map(|group| (group.label.as_str(), group.count))
            .collect::<Vec<_>>(),
        vec![("Groupe Astréa", 3), ("Nova Digital", 2)]
    );
    assert_eq!(par_entreprise[0].key, astrea.to_string());

    // Le filtre de statut s'applique : le refus d'Astréa ne compte plus.
    let sans_refus = ApplicationFilter {
        status: vec![ApplicationStatus::Pending, ApplicationStatus::Interview],
        ..ApplicationFilter::default()
    };
    let par_contrat = repo
        .groups(&sans_refus, ApplicationGrouping::Contract)
        .unwrap();
    assert_eq!(
        par_contrat
            .iter()
            .map(|group| (group.key.as_str(), group.count))
            .collect::<Vec<_>>(),
        vec![("CDI", 3), ("CDD", 1)]
    );
}
