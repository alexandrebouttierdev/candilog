//! Notifications non intrusives.
//!
//! Le toast flotte en bas à droite : il ne pousse jamais le contenu et n'ouvre
//! jamais de dialogue pour une information mineure.

use super::button as controls;
use super::icon::{self, Icon};
use super::typo;
use crate::ui::theme::metrics::{elevation, radius, size, space, stroke};
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Shadow, Theme, Vector};

/// Nature d'une notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Kind {
    /// Information factuelle.
    #[default]
    Info,
    /// Opération réussie.
    Success,
    /// Attention requise.
    Warning,
    /// Échec d'une opération.
    Error,
}

impl Kind {
    /// Ton sémantique associé.
    #[must_use]
    pub const fn tone(self) -> Tone {
        match self {
            Self::Info => Tone::Info,
            Self::Success => Tone::Success,
            Self::Warning => Tone::Warning,
            Self::Error => Tone::Danger,
        }
    }

    /// Icône associée.
    #[must_use]
    pub const fn icon(self) -> Icon {
        match self {
            Self::Info => Icon::Info,
            Self::Success => Icon::Check,
            Self::Warning | Self::Error => Icon::Alert,
        }
    }

    /// Déduit la nature d'un message métier sans imposer de refonte des
    /// services : un message d'erreur ne doit pas s'afficher en vert.
    #[must_use]
    pub fn infer(message: &str) -> Self {
        let lowered = message.to_lowercase();
        const FAILURES: [&str; 8] = [
            "erreur",
            "impossible",
            "échec",
            "echec",
            "invalide",
            "introuvable",
            "refus",
            "timeout",
        ];
        const CAUTIONS: [&str; 4] = ["attention", "annulé", "annule", "expiré"];
        if FAILURES.iter().any(|needle| lowered.contains(needle)) {
            Self::Error
        } else if CAUTIONS.iter().any(|needle| lowered.contains(needle)) {
            Self::Warning
        } else {
            Self::Success
        }
    }
}

/// Toast ancré en bas à droite du plan de travail.
pub fn toast<'a, Message: Clone + 'a>(
    message: String,
    kind: Kind,
    on_dismiss: Message,
) -> Element<'a, Message> {
    let tone = kind.tone();
    let card = container(
        row![
            icon::toned(kind.icon(), tone),
            typo::body(message),
            Space::with_width(space::MD),
            controls::icon_action(Icon::Close, "Masquer", on_dismiss),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .max_width(size::TOAST)
    .padding([space::MD, space::LG])
    .style(move |theme: &Theme| {
        let palette = tokens(theme);
        container::Style {
            background: Some(Background::Color(palette.raised)),
            text_color: Some(palette.text),
            border: Border {
                color: tone.edge(&palette),
                width: stroke::HAIRLINE,
                radius: radius::PANEL.into(),
            },
            shadow: Shadow {
                color: palette.shadow,
                offset: Vector::new(0.0, elevation::TOAST_OFFSET),
                blur_radius: elevation::TOAST_BLUR,
            },
        }
    });

    column![
        Space::with_height(Length::Fill),
        row![Space::with_width(Length::Fill), card],
    ]
    .padding(space::XXL)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::Kind;
    use crate::ui::theme::Tone;

    #[test]
    fn une_erreur_ne_s_affiche_jamais_comme_un_succes() {
        assert_eq!(Kind::infer("Erreur : provider IA injoignable"), Kind::Error);
        assert_eq!(Kind::infer("Impossible d'ouvrir la base"), Kind::Error);
        assert_eq!(Kind::infer("Fichier introuvable"), Kind::Error);
        assert_eq!(Kind::infer("Échec de l'export PDF"), Kind::Error);
    }

    #[test]
    fn un_avertissement_est_distingue_d_une_erreur() {
        assert_eq!(Kind::infer("Opération annulée"), Kind::Warning);
        assert_eq!(Kind::infer("Attention : profil incomplet"), Kind::Warning);
    }

    #[test]
    fn une_operation_reussie_reste_positive() {
        assert_eq!(Kind::infer("Candidature enregistrée"), Kind::Success);
        assert_eq!(Kind::infer("Backup exporté"), Kind::Success);
    }

    #[test]
    fn chaque_nature_porte_un_ton_et_une_icone_propres() {
        let kinds = [Kind::Info, Kind::Success, Kind::Warning, Kind::Error];
        for (index, kind) in kinds.iter().enumerate() {
            for other in &kinds[index + 1..] {
                assert_ne!(kind.tone(), other.tone());
            }
        }
        assert_eq!(Kind::Error.tone(), Tone::Danger);
        assert_eq!(Kind::default(), Kind::Info);
    }

    #[test]
    fn la_detection_ignore_la_casse_et_les_accents_courants() {
        assert_eq!(Kind::infer("ERREUR SQLITE"), Kind::Error);
        assert_eq!(Kind::infer("Echec du téléchargement"), Kind::Error);
    }
}
