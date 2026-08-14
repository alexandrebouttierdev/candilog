//! Cas de test isolé.

use super::*;

#[test]
fn generation_terminee_laisse_place_aux_suggestions() {
    assert_eq!(
        panel_footer_state(true, false, true),
        PanelFooterState::None
    );
}
