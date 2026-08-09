//! Cas de test isolé.

use super::*;

#[test]
fn test_ignore_les_entrees_sans_contenu_utile() {
    let profile = parse(
        r#"{"experiences":[{"title":"","company":""}],"projects":[{"name":""}],"languages":[{"name":"","level":"C1"}]}"#,
    );
    assert!(profile.experiences.is_empty());
    assert!(profile.projects.is_empty());
    assert!(profile.languages.is_empty());
}
