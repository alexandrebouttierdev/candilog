//! Cas de test isolé.

use super::*;

#[test]
fn test_cancel_generation_inconnue_est_sans_effet() {
    let state = AppState::new().unwrap();
    state.cancel_generation("inexistant"); // ne doit pas paniquer
}
