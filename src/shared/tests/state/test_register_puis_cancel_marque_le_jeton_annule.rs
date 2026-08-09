//! Cas de test isolé.

use super::*;

#[test]
fn test_register_puis_cancel_marque_le_jeton_annule() {
    let state = AppState::new().unwrap();
    let token = state.register_generation("g1");
    assert!(!token.is_cancelled());
    state.cancel_generation("g1");
    assert!(token.is_cancelled());
}
