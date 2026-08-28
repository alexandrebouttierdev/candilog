//! Cas de test isolé.

use super::*;

/// Le canal est un texte libre en base, sans contrainte `CHECK` : rien n'empêcherait d'y
/// stocker une chaîne vide, qui s'afficherait comme une pastille muette au calendrier.
#[test]
fn test_canal_vide_est_refuse() {
    let service = FollowUpService::new(StubRepo);
    let mut input = new("2026-08-27");
    input.channel = "  ".into();

    assert!(matches!(
        service.create(&input),
        Err(AppError::Validation(_))
    ));
}
