//! Cas de test isolé.

use super::*;

/// Le site web est saisi librement puis ouvert d'un clic depuis la fiche : un schéma autre
/// que HTTP(S) y ferait exécuter du code au lieu d'ouvrir une page.
#[test]
fn test_creer_refuse_un_site_web_non_http() {
    let service = EntrepriseService::new(StubRepo);
    let mut input = nouvelle("Nova Digital");
    input.site_web = Some("javascript:alert(1)".into());

    let resultat = service.creer(&input);

    assert!(matches!(resultat, Err(AppError::Validation(_))));
}
