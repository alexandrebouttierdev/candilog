//! Cas de test isolé.

use super::*;

/// La modification passe par le même contrôle que la création : sans cela, une candidature
/// valide à la création pourrait être rendue invalide par une édition.
#[test]
fn test_modifier_valide_les_memes_regles_que_creer() {
    let service = ApplicationService::new(StubRepo);
    let mut input = new("Développeur");
    input.sent_date = "20-08-2026".into();

    let resultat = service.update(uuid::Uuid::nil(), &input);

    assert!(matches!(resultat, Err(AppError::Validation(_))));
}
