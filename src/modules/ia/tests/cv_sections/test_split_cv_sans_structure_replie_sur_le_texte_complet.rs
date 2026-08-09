//! Cas de test isolé.

use super::*;

#[test]
fn test_split_cv_sans_structure_replie_sur_le_texte_complet() {
    let flat = "Ada Lovelace, ingénieure Rust chez ACME depuis 2022, MSc Cambridge.";
    let sections = split_cv(flat);
    assert_eq!(sections.identity, flat);
    assert_eq!(sections.history, flat);
    assert_eq!(sections.skills, flat);
    assert_eq!(sections.portfolio, flat);
}
