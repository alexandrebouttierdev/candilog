//! Tons sémantiques : la couleur ne sert qu'à répondre à une question.
//!
//! Un composant ne choisit jamais une couleur ; il choisit un [`Tone`], que le
//! thème actif résout. Aucune information ne repose sur la couleur seule : un
//! ton est toujours accompagné d'un libellé et, pour les statuts, d'une forme.

use super::tokens::{tokens, Tokens};
use iced::{Color, Theme};

/// Intention sémantique d'un élément coloré.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tone {
    /// Aucune intention : l'élément reste dans les gris de l'interface.
    #[default]
    Neutral,
    /// Action principale, sélection, élément actif.
    Accent,
    /// Résultat positif ou étape franchie.
    Success,
    /// Attention requise sans gravité.
    Warning,
    /// Échec, refus ou action destructive.
    Danger,
    /// Information factuelle.
    Info,
    /// Violet des statuts Entretien et des événements de calendrier.
    Violet,
}

impl Tone {
    /// Couleur du texte et des glyphes portant ce ton.
    #[must_use]
    pub fn color(self, palette: &Tokens) -> Color {
        match self {
            Self::Neutral => palette.text_secondary,
            Self::Accent => palette.accent,
            Self::Success => palette.success,
            Self::Warning => palette.warning,
            Self::Danger => palette.danger,
            Self::Info => palette.info,
            Self::Violet => palette.violet,
        }
    }

    /// Fond discret d'un jeton portant ce ton.
    #[must_use]
    pub fn surface(self, palette: &Tokens) -> Color {
        match self {
            Self::Neutral => palette.neutral_tint,
            Self::Accent => palette.accent_tint,
            Self::Success => palette.success_tint,
            Self::Warning => palette.warning_tint,
            Self::Danger => palette.danger_tint,
            Self::Info | Self::Violet => {
                // Les tons sans teinte dédiée restent sur un fond translucide.
                let base = self.color(palette);
                Color {
                    a: if palette.is_dark { 0.14 } else { 0.10 },
                    ..base
                }
            }
        }
    }

    /// Filet d'un jeton portant ce ton.
    #[must_use]
    pub fn edge(self, palette: &Tokens) -> Color {
        if self == Self::Neutral {
            return palette.border;
        }
        let base = self.color(palette);
        Color {
            a: if palette.is_dark { 0.32 } else { 0.26 },
            ..base
        }
    }

    /// Résout la couleur du ton directement depuis le thème actif.
    #[must_use]
    pub fn resolve(self, theme: &Theme) -> Color {
        self.color(&tokens(theme))
    }
}

/// Forme du marqueur accompagnant un ton, pour ne jamais dépendre de la couleur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    /// Cercle creux : état d'attente.
    Hollow,
    /// Demi-cercle : action engagée, en cours.
    Half,
    /// Cercle plein : étape franchie.
    Solid,
    /// Cercle barré : issue négative.
    Barred,
}

#[cfg(test)]
mod tests {
    use super::{Marker, Tone};
    use crate::ui::theme::tokens::{DAY, NIGHT};

    #[test]
    fn ton_neutre_reste_dans_les_gris() {
        for palette in [NIGHT, DAY] {
            assert_eq!(Tone::Neutral.color(&palette), palette.text_secondary);
            assert_eq!(Tone::Neutral.surface(&palette), palette.sunken);
            assert_eq!(Tone::Neutral.edge(&palette), palette.border);
        }
    }

    #[test]
    fn tons_semantiques_portent_une_teinte_opaque() {
        for palette in [NIGHT, DAY] {
            for tone in [Tone::Accent, Tone::Success, Tone::Warning, Tone::Danger] {
                let surface = tone.surface(&palette);
                assert_eq!(surface.a, 1.0, "la teinte doit être opaque");
                assert_ne!(
                    surface,
                    tone.color(&palette),
                    "la teinte ne doit pas se confondre avec le ton lui-même"
                );
            }
        }
    }

    #[test]
    fn tons_sont_deux_a_deux_distincts() {
        let palette = NIGHT;
        let tones = [
            Tone::Accent,
            Tone::Success,
            Tone::Warning,
            Tone::Danger,
            Tone::Info,
        ];
        for (index, tone) in tones.iter().enumerate() {
            for other in &tones[index + 1..] {
                assert_ne!(tone.color(&palette), other.color(&palette));
            }
        }
    }

    #[test]
    fn ton_par_defaut_est_neutre() {
        assert_eq!(Tone::default(), Tone::Neutral);
    }

    #[test]
    fn marqueurs_couvrent_les_quatre_etats_de_pipeline() {
        let markers = [Marker::Hollow, Marker::Half, Marker::Solid, Marker::Barred];
        assert_eq!(markers.len(), 4);
        for (index, marker) in markers.iter().enumerate() {
            for other in &markers[index + 1..] {
                assert_ne!(marker, other);
            }
        }
    }

    #[test]
    fn tons_se_resolvent_depuis_le_theme_actif() {
        assert_eq!(
            Tone::Accent.resolve(&crate::ui::theme::dark()),
            NIGHT.accent
        );
        assert_eq!(Tone::Accent.resolve(&crate::ui::theme::light()), DAY.accent);
    }
}
