//! Cas de test isolé.

use super::*;

#[test]
fn generation_en_cours_affiche_la_progression() {
    assert_eq!(
        panel_footer_state(true, true, false),
        PanelFooterState::Generating
    );
}
