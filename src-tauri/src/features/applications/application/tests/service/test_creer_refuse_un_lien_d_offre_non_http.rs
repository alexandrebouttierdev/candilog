//! Cas de test isolé.

use super::*;

/// Le lien de l'offre est ouvert d'un clic depuis la fiche : un schéma autre que HTTP(S)
/// y ferait exécuter du code au lieu d'ouvrir une page.
#[test]
fn test_creer_refuse_un_lien_d_offre_non_http() {
    let service = ApplicationService::new(StubRepo::default());
    let mut input = new("Développeur");
    input.job_url = Some("javascript:alert(1)".into());

    assert!(matches!(
        service.create(&input),
        Err(AppError::Validation(_))
    ));
}
