//! Cas de test isolé.

use super::*;

#[test]
fn test_score_keywords_densite_partielle_donne_50() {
    let p = Profile {
        personal: PersonalInfo {
            summary: Some("Expert Kubernetes".into()),
            ..PersonalInfo::default()
        },
        ..Profile::default()
    };
    let o = offer(&[], &["kubernetes", "terraform"], None);
    let s = score(&p, &o);
    assert_eq!(s.ats, 50);
}
