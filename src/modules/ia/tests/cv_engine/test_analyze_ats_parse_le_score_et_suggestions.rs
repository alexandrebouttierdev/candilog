//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_analyze_ats_parse_le_score_et_suggestions() {
    // Réponse sans `recommandations` : le champ doit rester vide (rétro-compatibilité serde default).
    let body = r#"{"score":82,"suggestions":["Ajouter Kubernetes","Mettre en avant Rust"]}"#;
    let a = engine(vec![body])
        .analyze_ats(
            &crate::modules::ia::cv_model::GeneratedCv::default(),
            &ParsedOffer::default(),
        )
        .await
        .unwrap();
    assert_eq!(a.score, 82);
    assert_eq!(a.suggestions.len(), 2);
    assert!(a.recommandations.is_empty());
}
