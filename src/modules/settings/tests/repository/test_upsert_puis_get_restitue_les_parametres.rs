//! Cas de test isolé.

use super::*;

#[test]
fn test_upsert_puis_get_restitue_les_parametres() {
    let repo = repo();
    let parametres = AppSettings {
        langue: "en".into(),
        ..AppSettings::default()
    };
    repo.upsert(&parametres).unwrap();
    assert_eq!(repo.get().unwrap().langue, "en");
}
