//! Cas de test isolé.

use super::*;

#[test]
fn analyse_seule_propose_la_generation() {
    assert_eq!(
        panel_footer_state(true, false, false),
        PanelFooterState::ProposeGeneration
    );
}
