//! Cas de test isolé.

use super::*;

#[test]
fn l_ordre_d_apparition_est_preserve() {
    let analysis = score(&["C", "Rust", "C++"], &["Kafka", "SQL", "Kafka"]);
    assert_eq!(present_skills(&analysis), owned(&["C", "Rust", "C++"]));
    assert_eq!(missing_skills(&analysis), owned(&["Kafka", "SQL"]));
}
