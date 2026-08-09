//! Cas de test isolé.

use super::*;

#[test]
fn test_dedoublonne_les_competences_insensible_a_la_casse() {
    let profile = parse(
        r#"{"skills":[{"name":"Rust"},{"name":"  rust "},{"name":"TypeScript"},{"name":""}]}"#,
    );
    assert_eq!(profile.skills.len(), 2);
    assert_eq!(profile.skills[0].name, "Rust");
    assert_eq!(profile.skills[1].name, "TypeScript");
}
