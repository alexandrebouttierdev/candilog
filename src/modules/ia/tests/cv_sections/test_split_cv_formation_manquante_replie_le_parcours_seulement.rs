//! Cas de test isolé.

use super::*;

#[test]
fn test_split_cv_formation_manquante_replie_le_parcours_seulement() {
    // Expériences + compétences détectées, mais pas de section formation :
    // le parcours reçoit le texte complet, les compétences restent ciblées.
    let cv = "Ada\n\nExpérience\nDev ACME\n\nCompétences\nRust";
    let sections = split_cv(cv);
    assert_eq!(sections.history, cv);
    assert!(sections.skills.contains("Rust"));
    assert!(!sections.skills.contains("Dev ACME"));
}
