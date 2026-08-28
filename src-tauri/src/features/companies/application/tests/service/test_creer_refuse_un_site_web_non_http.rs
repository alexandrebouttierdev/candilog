//! Cas de test isolé.

use super::*;

/// Le site web est saisi librement puis ouvert d'un clic depuis la fiche : un schéma autre
/// que HTTP(S) y ferait exécuter du code au lieu d'ouvrir une page.
#[test]
fn test_creer_refuse_un_site_web_non_http() {
    let service = CompanyService::new(StubRepo);
    let mut input = new("Nova Digital");
    input.website = Some("javascript:alert(1)".into());

    let resultat = service.create(&input);

    assert!(matches!(resultat, Err(AppError::Validation(_))));
}
