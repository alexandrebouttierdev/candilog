//! Cas de test isolé.

use super::*;

#[test]
fn test_to_command_error_retourne_message_lisible() {
    let err = AppError::NotFound("settings".into());
    assert_eq!(err.to_command_error(), "Introuvable : settings");
}
