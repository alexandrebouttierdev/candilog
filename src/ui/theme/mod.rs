//! Système visuel de Candilog.
//!
//! Les deux thèmes sont définis indépendamment l'un de l'autre : `Candilog
//! Jour` n'est pas l'inverse de `Candilog Nuit`. Tout le reste de
//! l'application ne connaît que les jetons, les tons et les styles exposés ici.

pub mod color;
pub mod layout;
pub mod metrics;
pub mod styles;
pub mod tokens;
pub mod typography;

pub use color::{Marker, Tone};
pub use layout::Layout;
pub use tokens::{tokens, Tokens};

use iced::{theme, Theme};

/// Thème sombre « Candilog Nuit ».
#[must_use]
pub fn dark() -> Theme {
    Theme::custom(
        "Candilog Nuit".into(),
        theme::Palette {
            background: tokens::NIGHT.canvas,
            text: tokens::NIGHT.text,
            primary: tokens::NIGHT.accent,
            success: tokens::NIGHT.success,
            danger: tokens::NIGHT.danger,
        },
    )
}

/// Thème clair « Candilog Jour ».
#[must_use]
pub fn light() -> Theme {
    Theme::custom(
        "Candilog Jour".into(),
        theme::Palette {
            background: tokens::DAY.canvas,
            text: tokens::DAY.text,
            primary: tokens::DAY.accent,
            success: tokens::DAY.success,
            danger: tokens::DAY.danger,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{dark, light, tokens};

    #[test]
    fn les_deux_themes_portent_un_nom_distinct() {
        assert_ne!(dark().to_string(), light().to_string());
    }

    #[test]
    fn chaque_theme_resout_ses_propres_jetons() {
        assert!(tokens(&dark()).is_dark);
        assert!(!tokens(&light()).is_dark);
    }

    #[test]
    fn le_mode_clair_n_est_pas_une_inversion_du_mode_sombre() {
        let night = tokens(&dark());
        let day = tokens(&light());
        // Une inversion mécanique produirait des écarts de surface identiques.
        let night_gap = night.panel.r - night.canvas.r;
        let day_gap = day.canvas.r - day.panel.r;
        assert!((night_gap + day_gap).abs() > 0.005);
    }
}
