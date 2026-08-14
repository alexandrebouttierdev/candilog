//! Cas de test isolé.

use super::*;

#[test]
fn chaque_colonne_porte_un_libelle_distinct() {
    let labels: std::collections::BTreeSet<_> = PIPELINE
        .iter()
        .map(|status| column_label(*status))
        .collect();
    assert_eq!(labels.len(), PIPELINE.len());
}
