//! Cas de test isolé.

use super::*;

#[test]
fn test_split_cv_projets_absents_replie_le_portfolio() {
    let cv = "Ada\n\nExpérience\nDev ACME\n\nFormation\nMSc";
    let sections = split_cv(cv);
    assert_eq!(sections.portfolio, cv);
}
