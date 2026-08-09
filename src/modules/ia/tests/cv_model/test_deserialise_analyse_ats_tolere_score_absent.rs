//! Cas de test isolé.

use super::*;

#[test]
fn test_deserialise_analyse_ats_tolere_score_absent() {
    let analysis: AtsAnalysis =
        serde_json::from_str(r#"{"suggestions":["Détailler les missions"]}"#).unwrap();

    assert_eq!(analysis.score, 0);
    assert_eq!(analysis.suggestions, vec!["Détailler les missions"]);
}
