//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_analyze_ats_parse_les_recommandations_structurees() {
    let body = r#"{"score":74,"suggestions":[],"recommandations":[{"section":"resume","texte_original":"Dev","texte_propose":"Dev senior orienté offre","impact":8}]}"#;
    let a = engine(vec![body])
        .analyze_ats(
            &crate::modules::ia::cv_model::GeneratedCv::default(),
            &ParsedOffer::default(),
        )
        .await
        .unwrap();
    assert_eq!(a.recommandations.len(), 1);
    assert_eq!(a.recommandations[0].section, "resume");
    assert_eq!(a.recommandations[0].impact, 8);
    assert_eq!(
        a.recommandations[0].texte_propose,
        "Dev senior orienté offre"
    );
}
