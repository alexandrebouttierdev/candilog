//! Cas de test isolé.

use super::*;

/// Le Kanban affiche les quatre colonnes en permanence : si la répartition tenait compte du
/// filtre de statut, sélectionner « Entretien » viderait les trois autres en-têtes.
#[test]
fn test_repartition_ignore_le_filtre_de_statut() {
    let (repo, entreprise_id) = contexte();
    for statut in [StatutCandidature::EnAttente, StatutCandidature::Entretien] {
        let mut input = entree(entreprise_id, "Développeur", "2026-08-20");
        input.statut = statut;
        repo.create(&input).unwrap();
    }

    let repartition = repo
        .repartition(&FiltreCandidatures {
            statut: Some(StatutCandidature::Entretien),
            ..FiltreCandidatures::default()
        })
        .unwrap();

    assert_eq!(repartition.en_attente, 1);
    assert_eq!(repartition.entretien, 1);
}
