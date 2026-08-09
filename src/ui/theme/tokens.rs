//! Jetons visuels résolus depuis le thème Iced actif.
//!
//! Un seul point de vérité pour les surfaces, les filets, les textes et les
//! accents. Les deux thèmes sont décrits indépendamment : le mode clair n'est
//! pas une inversion mécanique du mode sombre.

use iced::{Color, Theme};

/// Construit une couleur opaque depuis une notation hexadécimale compacte.
const fn hex(value: u32) -> Color {
    Color {
        r: ((value >> 16) & 0xFF) as f32 / 255.0,
        g: ((value >> 8) & 0xFF) as f32 / 255.0,
        b: (value & 0xFF) as f32 / 255.0,
        a: 1.0,
    }
}

/// Applique une opacité à une couleur de la palette.
#[must_use]
pub const fn alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

/// Ensemble complet des couleurs d'un thème Candilog.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tokens {
    /// Vrai pour le thème sombre, utilisé par les rares réglages asymétriques.
    pub is_dark: bool,

    /// Barre latérale, barre d'état : la structure de la fenêtre.
    pub chrome: Color,
    /// Plan de travail : fond de la zone de contenu.
    pub canvas: Color,
    /// Surface d'un panneau de données.
    pub panel: Color,
    /// Creux : champs, en-têtes de table, plan de travail d'un document.
    pub sunken: Color,
    /// Surélevé : menus, dialogues, drawers, ligne survolée.
    pub raised: Color,
    /// Survol d'un élément interactif posé sur `panel`.
    pub hover: Color,

    /// Filet standard entre deux surfaces de même niveau.
    pub border: Color,
    /// Filet d'un contrôle, d'un en-tête de table ou d'un panneau actif.
    pub border_strong: Color,

    /// Texte principal.
    pub text: Color,
    /// Texte de métadonnée.
    pub text_secondary: Color,
    /// Texte désactivé ou indicatif.
    pub text_muted: Color,

    /// Accent pétrole pour les textes, icônes et filets actifs.
    pub accent: Color,
    /// Accent pétrole pour les surfaces pleines.
    pub accent_fill: Color,
    /// Accent pétrole au survol d'une surface pleine.
    pub accent_hover: Color,
    /// Texte posé sur une surface d'accent pleine.
    pub on_accent: Color,
    /// Fond de sélection discret.
    pub selection: Color,

    /// Réussite.
    pub success: Color,
    /// Avertissement.
    pub warning: Color,
    /// Erreur ou action destructive.
    pub danger: Color,
    /// Information neutre.
    pub info: Color,

    /// Papier d'un aperçu de document, constant dans les deux thèmes.
    pub paper: Color,
    /// Encre principale d'un aperçu de document.
    pub paper_ink: Color,
    /// Encre secondaire d'un aperçu de document.
    pub paper_ink_muted: Color,
    /// Filet d'un aperçu de document.
    pub paper_rule: Color,

    /// Voile posé sous une modale.
    pub scrim: Color,
    /// Ombre de référence des éléments réellement flottants.
    pub shadow: Color,
}

/// Jetons du thème sombre « Candilog Nuit ».
pub const NIGHT: Tokens = Tokens {
    is_dark: true,

    chrome: hex(0x0F1413),
    canvas: hex(0x151A19),
    panel: hex(0x1D2422),
    sunken: hex(0x171D1C),
    raised: hex(0x242C2A),
    hover: hex(0x2A3331),

    border: hex(0x2A3330),
    border_strong: hex(0x3B4744),

    text: hex(0xEAF1EE),
    text_secondary: hex(0xA2B0AB),
    text_muted: hex(0x77857F),

    accent: hex(0x5FD1B2),
    accent_fill: hex(0x2E9E82),
    accent_hover: hex(0x37B896),
    on_accent: hex(0x05110E),
    selection: Color {
        a: 0.16,
        ..hex(0x5FD1B2)
    },

    success: hex(0x4FC98E),
    warning: hex(0xE0B15C),
    danger: hex(0xEC7A72),
    info: hex(0x6FAEF0),

    paper: hex(0xFBFAF7),
    paper_ink: hex(0x1A1D21),
    paper_ink_muted: hex(0x5C646D),
    paper_rule: hex(0xD8D6D0),

    scrim: Color {
        a: 0.58,
        ..hex(0x040A08)
    },
    shadow: Color {
        a: 0.46,
        ..hex(0x000000)
    },
};

/// Jetons du thème clair « Candilog Jour ».
pub const DAY: Tokens = Tokens {
    is_dark: false,

    chrome: hex(0xE6EBE9),
    canvas: hex(0xF0F4F2),
    panel: hex(0xFFFFFF),
    sunken: hex(0xF5F8F7),
    raised: hex(0xFFFFFF),
    hover: hex(0xE9EFED),

    border: hex(0xDCE4E1),
    border_strong: hex(0xBCC8C4),

    text: hex(0x131917),
    text_secondary: hex(0x55635E),
    text_muted: hex(0x7A8783),

    accent: hex(0x0C6E59),
    accent_fill: hex(0x0C6E59),
    accent_hover: hex(0x095847),
    on_accent: hex(0xFFFFFF),
    selection: Color {
        a: 0.10,
        ..hex(0x0C6E59)
    },

    success: hex(0x10794F),
    warning: hex(0x9A6408),
    danger: hex(0xC0392F),
    info: hex(0x2563C9),

    paper: hex(0xFFFFFF),
    paper_ink: hex(0x1A1D21),
    paper_ink_muted: hex(0x5C646D),
    paper_rule: hex(0xDFE2E6),

    scrim: Color {
        a: 0.26,
        ..hex(0x18211E)
    },
    shadow: Color {
        a: 0.22,
        ..hex(0x0E1614)
    },
};

/// Résout les jetons du thème Iced actif.
#[must_use]
pub fn tokens(theme: &Theme) -> Tokens {
    if theme.extended_palette().is_dark {
        NIGHT
    } else {
        DAY
    }
}

#[cfg(test)]
mod tests {
    use super::{hex, tokens, Tokens, DAY, NIGHT};
    use iced::Color;

    /// Luminance relative WCAG.
    fn luminance(color: Color) -> f32 {
        fn channel(value: f32) -> f32 {
            if value <= 0.039_28 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b)
    }

    fn contrast(foreground: Color, background: Color) -> f32 {
        let (a, b) = (luminance(foreground), luminance(background));
        let (light, dark) = if a > b { (a, b) } else { (b, a) };
        (light + 0.05) / (dark + 0.05)
    }

    #[test]
    fn hexadecimal_est_converti_en_canaux_normalises() {
        let color = hex(0x336699);
        assert!((color.r - 0.2).abs() < 0.01);
        assert!((color.g - 0.4).abs() < 0.01);
        assert!((color.b - 0.6).abs() < 0.01);
        assert!((color.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn texte_principal_respecte_le_contraste_courant() {
        for palette in [NIGHT, DAY] {
            assert!(
                contrast(palette.text, palette.panel) >= 4.5,
                "texte principal insuffisant"
            );
            assert!(
                contrast(palette.text, palette.canvas) >= 4.5,
                "texte principal insuffisant sur le plan de travail"
            );
        }
    }

    #[test]
    fn texte_secondaire_reste_lisible() {
        for palette in [NIGHT, DAY] {
            assert!(
                contrast(palette.text_secondary, palette.panel) >= 4.0,
                "texte secondaire insuffisant"
            );
        }
    }

    #[test]
    fn accent_plein_supporte_son_texte() {
        for palette in [NIGHT, DAY] {
            assert!(
                contrast(palette.on_accent, palette.accent_fill) >= 4.0,
                "texte sur accent insuffisant"
            );
        }
    }

    /// Moyenne sRGB : proche du noir, la luminance relative écrase des écarts
    /// pourtant bien visibles à l'écran.
    fn tint(color: Color) -> f32 {
        (color.r + color.g + color.b) / 3.0
    }

    #[test]
    fn chrome_se_distingue_du_plan_de_travail() {
        for palette in [NIGHT, DAY] {
            let separation = (tint(palette.chrome) - tint(palette.canvas)).abs();
            assert!(
                separation >= 0.012,
                "la barre latérale doit se détacher du contenu"
            );
        }
    }

    #[test]
    fn le_panneau_se_detache_du_plan_de_travail() {
        for palette in [NIGHT, DAY] {
            let separation = (tint(palette.panel) - tint(palette.canvas)).abs();
            assert!(
                separation >= 0.012,
                "un panneau doit se distinguer de son plan de travail"
            );
        }
    }

    #[test]
    fn surfaces_sont_hierarchisees() {
        assert!(luminance(NIGHT.chrome) < luminance(NIGHT.canvas));
        assert!(luminance(NIGHT.canvas) < luminance(NIGHT.panel));
        assert!(luminance(NIGHT.panel) < luminance(NIGHT.raised));

        assert!(luminance(DAY.chrome) < luminance(DAY.canvas));
        assert!(luminance(DAY.canvas) < luminance(DAY.panel));
    }

    #[test]
    fn themes_candilog_resolvent_leurs_jetons() {
        assert_eq!(tokens(&super::super::dark()), NIGHT);
        assert_eq!(tokens(&super::super::light()), DAY);
    }

    #[test]
    fn statuts_restent_distinguables_du_texte_courant() {
        for palette in [NIGHT, DAY] {
            let signals: [Color; 4] = [
                palette.success,
                palette.warning,
                palette.danger,
                palette.accent,
            ];
            for signal in signals {
                assert!(
                    contrast(signal, palette.panel) >= 3.0,
                    "signal sémantique insuffisamment contrasté"
                );
            }
        }
    }

    #[test]
    fn jetons_sont_copiables_sans_allocation() {
        let palette: Tokens = NIGHT;
        let copie = palette;
        assert_eq!(palette, copie);
    }

    /// La pastille inactive du rail (fond `panel`, filet `border`) doit rester
    /// visible sur le `chrome` du rail, sinon la forme carrée disparaît pour
    /// toutes les entrées sauf l'active.
    #[test]
    fn la_pastille_inactive_reste_visible_sur_le_rail() {
        for palette in [NIGHT, DAY] {
            assert_ne!(palette.panel, palette.chrome, "pastille noyée dans le rail");
            assert!(
                contrast(palette.panel, palette.chrome) > 1.05
                    || contrast(palette.border, palette.chrome) > 1.2,
                "pastille inactive indiscernable du rail"
            );
        }
    }

    /// L'icône reste lisible dans sa pastille, active comme inactive.
    #[test]
    fn l_icone_reste_lisible_dans_sa_pastille() {
        for palette in [NIGHT, DAY] {
            assert!(
                contrast(palette.text_secondary, palette.panel) >= 4.5,
                "icône inactive illisible"
            );
            assert!(
                contrast(palette.on_accent, palette.accent_fill) >= 4.5,
                "icône active illisible"
            );
        }
    }

    /// Les libellés de tuile se lisent sur le fond du rail, dans les deux états.
    #[test]
    fn les_libelles_de_tuile_se_lisent_sur_le_rail() {
        for palette in [NIGHT, DAY] {
            assert!(
                contrast(palette.accent, palette.chrome) >= 4.5,
                "libellé de tuile active illisible sur le rail"
            );
            assert!(
                contrast(palette.text_secondary, palette.chrome) >= 4.5,
                "libellé de tuile inactive illisible sur le rail"
            );
        }
    }
}
