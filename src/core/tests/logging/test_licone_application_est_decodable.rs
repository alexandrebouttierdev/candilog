//! L'icône embarquée doit rester décodable par Iced sur les plateformes desktop.

use super::*;

#[test]
fn test_licone_application_est_decodable() {
    assert!(
        icone_application().is_some(),
        "le PNG embarqué doit produire une icône de fenêtre valide"
    );
}
