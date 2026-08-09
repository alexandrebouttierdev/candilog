//! Cas de test isolé.

use super::*;

#[test]
fn test_save_nom_valide_delegue_au_depot_avec_nom_trim() {
    let svc = CvVersionService::new(MockRepo::default());
    let saved = svc.save("  CV Dev  ", &serde_json::json!({"a":1})).unwrap();
    assert_eq!(saved.name, "CV Dev");
    assert_eq!(svc.repo.created.lock().unwrap()[0].0, "CV Dev");
}
