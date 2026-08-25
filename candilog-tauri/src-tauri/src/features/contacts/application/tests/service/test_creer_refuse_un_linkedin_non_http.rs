//! Cas de test isolé.

use super::*;

/// Le champ LinkedIn du formulaire accepte une saisie libre : un `javascript:` collé depuis
/// une page malveillante ne doit pas être persisté, puisqu'il sera plus tard ouvert d'un clic.
#[test]
fn test_creer_refuse_un_linkedin_non_http() {
    let service = ContactService::new(StubRepo);
    let mut input = nouveau("Camille", "Rivet");
    input.linkedin = Some("javascript:alert(1)".into());

    let resultat = service.creer(&input);

    assert!(matches!(resultat, Err(AppError::Validation(_))));
}
