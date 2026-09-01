//! Cas de test isolé.

use super::*;

#[test]
fn enrichit_l_experience_du_lieu_et_de_la_periode() {
    let resume = build(&document());
    assert!(resume.experiences[0].location.as_deref() == Some("Rennes"));
    assert!(resume.experiences[0].period.contains("Juil."));
}
