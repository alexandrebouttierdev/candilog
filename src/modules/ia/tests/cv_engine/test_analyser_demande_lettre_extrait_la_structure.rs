//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_analyser_demande_lettre_extrait_la_structure() {
    let body = r#"{"companyName":"Orange","jobTitle":"TSSR","contractType":"professionnalisation","applicationType":"spontaneous","tone":null,"length":"short","keyPoints":["réseau"]}"#;
    let d = engine(vec![body])
        .analyser_demande_lettre("lettre spontanée chez Orange pour un contrat pro TSSR")
        .await
        .unwrap();
    assert_eq!(d.company_name.as_deref(), Some("Orange"));
    assert_eq!(d.contract_type.as_deref(), Some("professionnalisation"));
    assert_eq!(d.application_type.as_deref(), Some("spontaneous"));
    assert_eq!(d.key_points, vec!["réseau"]);
}
