//! Cas de test isolé.

use super::*;

#[test]
fn chaque_statut_porte_un_ton_et_une_forme_propres() {
    let mut tones = Vec::new();
    let mut markers = Vec::new();
    for status in PIPELINE {
        tones.push(status_tone(status));
        markers.push(status_marker(status));
    }
    for index in 0..PIPELINE.len() {
        for other in index + 1..PIPELINE.len() {
            assert_ne!(tones[index], tones[other], "tons de statut identiques");
            assert_ne!(
                markers[index], markers[other],
                "formes de statut identiques"
            );
        }
    }
}
