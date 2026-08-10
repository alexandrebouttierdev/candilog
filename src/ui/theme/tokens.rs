//! Jetons visuels résolus depuis le thème Iced actif.
//!
//! Un seul point de vérité pour les surfaces, les filets, les textes et les
//! accents. Les deux thèmes sont décrits indépendamment : le mode clair n'est
//! pas une inversion mécanique du mode sombre.

use iced::{Color, Theme};

/// Convertit une teinte `hsl(h s% l%)` (h en degrés, s et l en %) en couleur opaque.
///
/// Recette sRGB standard ; les valeurs sont celles du handoff candilog-desktop.
const fn hsl(h: f32, s: f32, l: f32) -> Color {
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
    /// Violet des statuts Entretien et des événements de calendrier.
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

/// Jetons du thème sombre (candilog-desktop `.dark`).
pub const NIGHT: Tokens = Tokens {
    is_dark: true,

    chrome: hsl(240.0, 14.0, 10.0), // --app
    canvas: hsl(240.0, 14.0, 10.0), // --app (fond ambiant)
    panel: hsl(240.0, 11.0, 17.0),  // --card
    sunken: hsl(240.0, 9.0, 22.0),  // --secondary / --muted
    raised: hsl(240.0, 11.0, 17.0), // --popover = --card
    hover: hsl(240.0, 9.0, 26.0),   // secondary éclairci (hover)

    border: hsl(240.0, 9.0, 26.0),        // --border
    border_strong: hsl(240.0, 9.0, 28.0), // --input

    text: hsl(240.0, 12.0, 95.0),          // --foreground
    text_secondary: hsl(240.0, 7.0, 68.0), // --muted-foreground
    text_muted: hsl(240.0, 7.0, 52.0),

    accent: hsl(245.0, 75.0, 70.0), // --primary
    accent_fill: hsl(245.0, 75.0, 70.0),
    accent_hover: hsl(245.0, 75.0, 66.0),
    on_accent: hsl(0.0, 0.0, 100.0), // --primary-foreground
    selection: Color {
        a: 0.14,
        ..hsl(245.0, 75.0, 70.0)
    },

    success: hsl(142.0, 71.0, 45.0), // --status-success
    warning: hsl(38.0, 92.0, 50.0),  // --status-warning
    danger: hsl(0.0, 84.0, 60.0),    // --status-danger
    info: hsl(199.0, 89.0, 48.0),    // --status-info
    violet: hsl(271.0, 91.0, 65.0),  // --status-violet

    paper: hsl(0.0, 0.0, 100.0),
    paper_ink: hsl(240.0, 10.0, 12.0),
    paper_ink_muted: hsl(240.0, 4.0, 40.0),
    paper_rule: hsl(240.0, 6.0, 88.0),

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

/// Jetons du thème clair (candilog-desktop `:root`).
pub const DAY: Tokens = Tokens {
    is_dark: false,

    chrome: hsl(240.0, 8.0, 95.0), // --app
    canvas: hsl(240.0, 8.0, 95.0), // --app (fond ambiant)
    panel: hsl(0.0, 0.0, 100.0),   // --card / --background
    sunken: hsl(240.0, 6.0, 95.0), // --secondary / --muted
    raised: hsl(0.0, 0.0, 100.0),  // --popover
    hover: hsl(240.0, 5.0, 93.0),  // secondary légèrement appuyé

    border: hsl(240.0, 6.0, 90.0),        // --border
    border_strong: hsl(240.0, 6.0, 88.0), // --input

    text: hsl(240.0, 10.0, 10.0),          // --foreground
    text_secondary: hsl(240.0, 4.0, 47.0), // --muted-foreground
    text_muted: hsl(240.0, 4.0, 58.0),

    accent: hsl(245.0, 52.0, 50.0), // --primary
    accent_fill: hsl(245.0, 52.0, 50.0),
    accent_hover: hsl(245.0, 52.0, 46.0),
    on_accent: hsl(0.0, 0.0, 100.0), // --primary-foreground
    selection: Color {
        a: 0.10,
        ..hsl(245.0, 52.0, 50.0)
    },

    success: hsl(142.0, 71.0, 45.0),
    warning: hsl(38.0, 92.0, 50.0),
    danger: hsl(0.0, 84.0, 60.0),
    info: hsl(199.0, 89.0, 48.0),
    violet: hsl(271.0, 91.0, 65.0),

    paper: hsl(0.0, 0.0, 100.0),
    paper_ink: hsl(240.0, 10.0, 12.0),
    paper_ink_muted: hsl(240.0, 4.0, 40.0),
    paper_rule: hsl(240.0, 6.0, 88.0),

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
        a: 0.18,
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
    fn la_palette_respecte_les_valeurs_du_handoff() {
        // Indigo primaire : hsl(245 52% 50%) → rgb(0.28, 0.24, 0.76) (clair),
        // hsl(245 75% 70%) → r ≈ 0.51 (sombre, plus clair que le jour).
        let day = DAY;
        let night = NIGHT;
        assert!((day.accent.r - 0.31).abs() < 0.05 && (day.accent.b - 0.76).abs() < 0.05);
        assert!(night.accent.r > 0.50, "indigo sombre trop sombre");
        // Statut violet : hsl(271 91% 65%) → g ≈ 0.33.
        assert!((day.violet.g - 0.33).abs() < 0.1);
        assert!(
            ecart_max(night.violet, day.violet) < 0.01,
            "statuts identiques clair/sombre"
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
