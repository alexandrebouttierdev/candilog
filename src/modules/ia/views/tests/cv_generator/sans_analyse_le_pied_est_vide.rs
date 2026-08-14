//! Cas de test isolé.

use super::*;

#[test]
fn sans_analyse_le_pied_est_vide() {
    assert_eq!(
        panel_footer_state(false, false, false),
        PanelFooterState::None
    );
    assert_eq!(
        panel_footer_state(false, true, false),
        PanelFooterState::None
    );
    assert_eq!(
        panel_footer_state(false, false, true),
        PanelFooterState::None
    );
}
