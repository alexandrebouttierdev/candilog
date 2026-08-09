//! Cas de test isolé.

use super::*;

#[test]
fn test_profile_is_empty_detecte_un_profil_vide() {
    assert!(profile_is_empty(&Profile::default()));
    let profile = parse(r#"{"skills":[{"name":"Go"}]}"#);
    assert!(!profile_is_empty(&profile));
}
