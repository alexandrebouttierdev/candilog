//! Cas de test isolé.

use super::*;

#[test]
fn l_attente_reste_neutre_pour_ne_pas_saturer_le_pipeline() {
    assert_eq!(status_tone(StatutCandidature::EnAttente), Tone::Neutral);
    assert_eq!(status_marker(StatutCandidature::EnAttente), Marker::Hollow);
}
