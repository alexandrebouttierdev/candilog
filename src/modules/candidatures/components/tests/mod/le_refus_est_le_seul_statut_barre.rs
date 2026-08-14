//! Cas de test isolé.

use super::*;

#[test]
fn le_refus_est_le_seul_statut_barre() {
    assert_eq!(status_marker(StatutCandidature::Refus), Marker::Barred);
    assert_eq!(status_tone(StatutCandidature::Refus), Tone::Danger);
}
