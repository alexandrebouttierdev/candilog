//! Cas de test isolé.

use super::*;

#[test]
fn test_experience_en_cours_efface_la_date_de_fin() {
    let profile = parse(
        r#"{"experiences":[{"title":"Dev","company":"ACME","start_date":"jan 2022","end_date":"2023","current":"oui"}]}"#,
    );
    assert_eq!(profile.experiences.len(), 1);
    assert_eq!(profile.experiences[0].start_date, "2022-01");
    assert!(profile.experiences[0].current);
    assert_eq!(profile.experiences[0].end_date, None);
}
