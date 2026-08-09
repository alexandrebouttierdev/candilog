//! Cas de test isolé.

use super::*;

#[test]
fn test_finish_generation_retire_du_registre() {
    let state = AppState::new().unwrap();
    let token = state.register_generation("g2");
    state.finish_generation("g2");
    // Retiré du registre : une annulation tardive ne peut plus l'atteindre.
    state.cancel_generation("g2");
    assert!(!token.is_cancelled());
}
