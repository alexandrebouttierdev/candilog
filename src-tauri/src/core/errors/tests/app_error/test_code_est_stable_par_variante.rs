//! Cas de test isolé.

use super::*;

/// Le frontend branche son comportement sur `code`, jamais sur le texte du message.
/// Renommer un code est donc une rupture de contrat IPC, que ce test rend visible.
#[test]
fn test_code_est_stable_par_variante() {
    let attendus = [
        (AppError::Validation(String::new()), "VALIDATION_ERROR"),
        (AppError::NotFound(String::new()), "NOT_FOUND"),
        (AppError::Database(String::new()), "DATABASE_ERROR"),
        (AppError::Http(String::new()), "HTTP_ERROR"),
        (
            AppError::Serialization(String::new()),
            "SERIALIZATION_ERROR",
        ),
        (AppError::Provider(String::new()), "PROVIDER_ERROR"),
        (AppError::Cancelled, "CANCELLED"),
    ];
    for (error, code) in attendus {
        assert_eq!(error.code(), code);
    }
}
