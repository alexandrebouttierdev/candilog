//! Cas de test isolé.

use super::*;

#[test]
fn test_split_cv_fragmente_chaque_section_au_bon_appel() {
    let sections = split_cv(CV);
    assert!(sections.identity.contains("ada@x.io"));
    assert!(!sections.identity.contains("MSc Maths"));
    assert!(sections.history.contains("Ingénieure, ACME"));
    assert!(sections.history.contains("MSc Maths"));
    assert!(!sections.history.contains("Rust, SQL"));
    assert!(sections.skills.contains("Rust, SQL"));
    assert!(sections.skills.contains("Anglais C1"));
    assert!(sections.portfolio.contains("Moteur analytique"));
    assert!(sections.portfolio.contains("AWS Architect"));
    assert!(!sections.portfolio.contains("Ingénieure"));
}
