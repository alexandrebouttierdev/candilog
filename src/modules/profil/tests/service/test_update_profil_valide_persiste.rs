//! Cas de test isolé.

use super::*;

#[test]
fn test_update_profil_valide_persiste() {
    let profil = Profile {
        personal: PersonalInfo {
            first_name: "Alex".into(),
            last_name: "B".into(),
            email: "a@b.co".into(),
            ..PersonalInfo::default()
        },
        ..Profile::default()
    };
    let saved = service().update(&profil).unwrap();
    assert_eq!(saved.personal.first_name, "Alex");
}
