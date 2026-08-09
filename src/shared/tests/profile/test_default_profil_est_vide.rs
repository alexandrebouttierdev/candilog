//! Cas de test isolé.

use super::*;

#[test]
fn test_default_profil_est_vide() {
    let p = Profile::default();
    assert!(p.personal.first_name.is_empty());
    assert!(p.experiences.is_empty());
    assert!(p.skills.is_empty());
}
