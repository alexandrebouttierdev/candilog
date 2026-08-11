//! Cas de test isolé.

use super::*;
use crate::modules::settings::model::ThemePref;

/// « Système » était traité exactement comme « Sombre » : un système en thème clair donnait
/// quand même du sombre. Il doit désormais suivre ce qui a été détecté.
#[test]
fn test_resoudre_suit_le_systeme_puis_retombe_sur_le_courant() {
    assert!(
        !resoudre(ThemePref::System, Some(false), true),
        "un système en clair doit donner le thème clair, même si l'app est en sombre"
    );
    assert!(resoudre(ThemePref::System, Some(true), false));

    // Système muet ou non interrogeable : on conserve le thème courant plutôt que d'imposer
    // une bascule arbitraire.
    assert!(resoudre(ThemePref::System, None, true));
    assert!(!resoudre(ThemePref::System, None, false));
}
