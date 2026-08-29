//! Cas de test isolé.

use super::*;

#[test]
fn enrichit_l_experience_du_lieu_et_de_la_periode() {
    let resume = build(&profile(), &generation());
    assert_eq!(resume.experiences.len(), 1);
    assert_eq!(
        resume.experiences[0].meta,
        "Rennes · Juil. 2019 – Oct. 2025"
    );
    assert_eq!(resume.experiences[0].bullets, vec!["Une description."]);
}
