//! Cas de test isolé.

use super::*;
use crate::modules::settings::model::ThemePref;

/// Un choix explicite prime sur le système, quel que soit ce que celui-ci annonce.
#[test]
fn test_resoudre_respecte_le_choix_explicite() {
    for systeme in [None, Some(true), Some(false)] {
        for courant in [true, false] {
            assert!(!resoudre(ThemePref::Light, systeme, courant));
            assert!(resoudre(ThemePref::Dark, systeme, courant));
        }
    }
}
