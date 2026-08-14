//! Jetons visuels résolus depuis le thème Iced actif.
//!
//! Un seul point de vérité pour les surfaces, les filets, les textes et les
//! accents. Les deux thèmes sont décrits indépendamment : le mode clair n'est
//! pas une inversion mécanique du mode sombre.

use iced::{Color, Theme};

/// Construit une couleur opaque depuis une valeur hexadécimale RGB.
const fn hex(value: u32) -> Color {
    Color::from_rgb(
        ((value >> 16) & 0xff) as f32 / 255.0,
        ((value >> 8) & 0xff) as f32 / 255.0,
        (value & 0xff) as f32 / 255.0,
    )
}

/// Convertit une teinte `hsl(h s% l%)` (h en degrés, s et l en %) en couleur opaque.
///
/// Recette sRGB standard ; les valeurs sont celles du handoff candilog-desktop.
pub const fn hsl(h: f32, s: f32, l: f32) -> Color {
    let s = (s / 100.0).clamp(0.0, 1.0);
    let l = (l / 100.0).clamp(0.0, 1.0);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = (h % 360.0 + 360.0) % 360.0 / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let (r1, g1, b1) = match h_prime as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    Color {
        r: r1 + m,
        g: g1 + m,
        b: b1 + m,
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

    /// Accent cognac pour les textes, icônes et filets actifs.
    pub accent: Color,
    /// Accent cognac pour les surfaces pleines.
    pub accent_fill: Color,
    /// Accent cognac au survol d'une surface pleine.
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
    /// Vert sauge secondaire des entretiens et événements.
    pub violet: Color,

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

/// Thème sombre « Graphite & Champagne ».
pub const NIGHT: Tokens = Tokens {
    is_dark: true,

    chrome: hex(0x0b0d0c),
    canvas: hex(0x111412),
    panel: hex(0x1e211f),
    sunken: hex(0x0f1210),
    raised: hex(0x272b27),
    hover: hex(0x2d322c),

    border: hex(0x383d37),
    border_strong: hex(0x50574e),

    text: hex(0xf4f1e8),
    text_secondary: hex(0xb2b0a8),
    text_muted: hex(0x7f827a),

    accent: hex(0xe5a24b),
    accent_fill: hex(0xd89035),
    accent_hover: hex(0xe3a04a),
    on_accent: hex(0x1b140b),
    selection: Color {
        a: 0.16,
        ..hex(0xe5a24b)
    },

    success: hex(0x55b978),
    warning: hex(0xd89a3d),
    danger: hex(0xe06f67),
    info: hex(0x71a49a),
    violet: hex(0x83aa71),

    paper: hex(0xfbfaf6),
    paper_ink: hex(0x1d211d),
    paper_ink_muted: hex(0x62675f),
    paper_rule: hex(0xd8d4ca),

    scrim: Color {
        a: 0.45,
        ..Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    },
    shadow: Color {
        a: 0.40,
        ..Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    },
};

/// Thème clair « Porcelaine & Cognac » : surfaces nettes, profondeur discrète et encre froide.
pub const DAY: Tokens = Tokens {
    is_dark: false,

    chrome: hex(0xf8f6f1),
    canvas: hex(0xf3f1ec),
    panel: hex(0xfffefd),
    sunken: hex(0xf0ede7),
    raised: hex(0xffffff),
    hover: hex(0xeae5dd),

    border: hex(0xddd7cd),
    border_strong: hex(0xc4b9aa),

    text: hex(0x17201d),
    text_secondary: hex(0x4f5a55),
    text_muted: hex(0x7b857f),

    accent: hex(0xa65322),
    accent_fill: hex(0xa65322),
    accent_hover: hex(0x8c4219),
    on_accent: hex(0xfffcf6),
    selection: Color {
        a: 0.12,
        ..hex(0xa65322)
    },

    success: hex(0x2e7a4d),
    warning: hex(0x95680f),
    danger: hex(0xb94842),
    info: hex(0x477a72),
    violet: hex(0x5e844f),

    paper: hex(0xffffff),
    paper_ink: hex(0x17201d),
    paper_ink_muted: hex(0x62675f),
    paper_rule: hex(0xd8d4ca),

    scrim: Color {
        a: 0.45,
        ..Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    },
    shadow: Color {
        a: 0.16,
        ..Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
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
    use super::{hsl, tokens, Tokens, DAY, NIGHT};
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

    /// Plus grand écart entre les canaux de deux couleurs.
    fn ecart_max(a: Color, b: Color) -> f32 {
        (a.r - b.r)
            .abs()
            .max((a.g - b.g).abs())
            .max((a.b - b.b).abs())
    }

    #[test]
    fn hsl_est_converti_en_canaux_normalises() {
        let (blanc, noir) = (hsl(0.0, 0.0, 100.0), hsl(0.0, 0.0, 0.0));
        assert_eq!((blanc.r, blanc.g, blanc.b), (1.0, 1.0, 1.0));
        assert_eq!((noir.r, noir.g, noir.b), (0.0, 0.0, 0.0));
        for (couleur, (r, g, b)) in [
            (hsl(0.0, 100.0, 50.0), (1.0, 0.0, 0.0)),
            (hsl(120.0, 100.0, 50.0), (0.0, 1.0, 0.0)),
            (hsl(240.0, 100.0, 50.0), (0.0, 0.0, 1.0)),
            (hsl(240.0, 0.0, 50.0), (0.5, 0.5, 0.5)),
        ] {
            assert!((couleur.r - r).abs() < 0.001);
            assert!((couleur.g - g).abs() < 0.001);
            assert!((couleur.b - b).abs() < 0.001);
            assert_eq!(couleur.a, 1.0);
        }
    }

    #[test]
    fn la_palette_respecte_la_direction_graphite_et_cognac() {
        let day = DAY;
        let night = NIGHT;
        assert!(day.accent.r > day.accent.g, "le cognac doit rester chaud");
        assert!(
            night.accent.r > night.accent.g,
            "le champagne doit rester chaud"
        );
        assert!(
            night.accent.g > day.accent.g,
            "accent nocturne pas assez lumineux"
        );
        assert!(
            day.canvas.r > day.canvas.b,
            "le thème clair doit rester ivoire"
        );
        assert!(
            ecart_max(night.violet, day.violet) > 0.05,
            "le vert sauge doit être adapté à chaque thème"
        );
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
                contrast(palette.on_accent, palette.accent_fill) >= 2.4,
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
    fn le_panneau_se_detache_du_fond_ambiant() {
        for palette in [NIGHT, DAY] {
            let separation = (tint(palette.panel) - tint(palette.canvas)).abs();
            assert!(
                separation >= 0.05,
                "le panneau doit se détacher du fond ambiant"
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
        assert!(luminance(NIGHT.canvas) < luminance(NIGHT.panel));
        assert!(luminance(NIGHT.panel) <= luminance(NIGHT.raised));

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
            let signals: [Color; 5] = [
                palette.success,
                palette.warning,
                palette.danger,
                palette.accent,
                palette.violet,
            ];
            for signal in signals {
                assert!(
                    contrast(signal, palette.panel) >= 2.0,
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

    /// L'icône reste lisible dans sa pastille, active comme inactive.
    #[test]
    fn l_icone_reste_lisible_dans_sa_pastille() {
        for palette in [NIGHT, DAY] {
            assert!(
                contrast(palette.text_secondary, palette.panel) >= 2.4,
                "icône inactive illisible"
            );
            assert!(
                contrast(palette.on_accent, palette.accent_fill) >= 2.4,
                "icône active illisible"
            );
        }
    }

    /// Les libellés de tuile se lisent sur le fond du rail, dans les deux états.
    #[test]
    fn les_libelles_de_tuile_se_lisent_sur_le_rail() {
        for palette in [NIGHT, DAY] {
            assert!(
                contrast(palette.accent, palette.chrome) >= 2.4,
                "libellé de tuile active illisible sur le rail"
            );
            assert!(
                contrast(palette.text_secondary, palette.chrome) >= 2.4,
                "libellé de tuile inactive illisible sur le rail"
            );
        }
    }
}
