//! Cas de test isolé.

use super::*;

#[test]
fn test_repartition_compte_les_quatre_statuts() {
    let (repo, entreprise_id) = contexte();
    for statut in [
        StatutCandidature::EnAttente,
        StatutCandidature::EnAttente,
        StatutCandidature::Entretien,
        StatutCandidature::Refus,
    ] {
        let mut input = entree(entreprise_id, "Développeur", "2026-08-20");
        input.statut = statut;
        repo.create(&input).unwrap();
    }

    let repartition = repo.repartition(&FiltreCandidatures::default()).unwrap();

    assert_eq!(repartition.en_attente, 2);
    assert_eq!(repartition.relancee, 0);
    assert_eq!(repartition.entretien, 1);
    assert_eq!(repartition.refus, 1);
}
