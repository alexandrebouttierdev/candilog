//! Cas de test isolé.

use super::*;

#[test]
fn test_le_reload_marque_l_application_initialisee() {
    let app = app_de_test();
    assert!(app.initialized, "le chargement initial a réussi");
    assert!(app.fatal_error.is_none());
    assert!(
        app.notification.is_none(),
        "un chargement complet ne signale rien"
    );
}
