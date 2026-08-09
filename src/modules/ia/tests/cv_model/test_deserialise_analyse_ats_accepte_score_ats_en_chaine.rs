//! Cas de test isolé.

use super::*;

#[test]
fn test_deserialise_analyse_ats_accepte_score_ats_en_chaine() {
    let analysis: AtsAnalysis =
        serde_json::from_str(r#"{"score_ats":"87%","suggestions":["Ajouter Go"]}"#).unwrap();

    assert_eq!(analysis.score, 87);
    assert_eq!(analysis.suggestions, vec!["Ajouter Go"]);
}
