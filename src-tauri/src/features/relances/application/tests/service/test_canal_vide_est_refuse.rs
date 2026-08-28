//! Cas de test isolé.

use super::*;

/// Le canal est un texte libre en base, sans contrainte `CHECK` : rien n'empêcherait d'y
/// stocker une chaîne vide, qui s'afficherait comme une pastille muette au calendrier.
#[test]
fn test_canal_vide_est_refuse() {
    let service = RelanceService::new(StubRepo);
    let mut input = nouvelle("2026-08-27");
    input.type_relance = "  ".into();

    assert!(matches!(
        service.creer(&input),
        Err(AppError::Validation(_))
    ));
}
