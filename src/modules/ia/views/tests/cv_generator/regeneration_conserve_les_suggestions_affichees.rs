//! Cas de test isolé.

use super::*;

#[test]
fn regeneration_conserve_les_suggestions_affichees() {
    assert_eq!(panel_footer_state(true, true, true), PanelFooterState::None);
}
