//! Cas de test isolé.

use super::*;

#[test]
fn le_pipeline_est_parcourable_dans_les_deux_sens() {
    assert_eq!(
        next_status(StatutCandidature::EnAttente),
        Some(StatutCandidature::Relancee)
    );
    assert_eq!(next_status(StatutCandidature::Refus), None);
    assert_eq!(previous_status(StatutCandidature::EnAttente), None);
    assert_eq!(
        previous_status(StatutCandidature::Entretien),
        Some(StatutCandidature::Relancee)
    );
}
