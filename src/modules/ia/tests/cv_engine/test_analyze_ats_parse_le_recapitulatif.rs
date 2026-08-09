//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_analyze_ats_parse_le_recapitulatif() {
    let body = r#"{"score":82,"recap":"Le profil correspond bien à l'offre grâce à Rust. Il doit mieux détailler son expérience Kubernetes.","suggestions":[]}"#;
    let analysis = engine(vec![body])
        .analyze_ats(
            &crate::modules::ia::cv_model::GeneratedCv::default(),
            &ParsedOffer::default(),
        )
        .await
        .unwrap();

    assert!(analysis.recap.contains("correspond bien"));
    assert!(analysis.recap.contains("Kubernetes"));
}
