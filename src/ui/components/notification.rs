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
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme, Vector};

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

    /// Nature d'une notification portant une erreur, **dérivée de la variante** et non
    /// devinée à partir du texte.
    ///
    /// La version précédente recherchait huit mots-clés français dans le message et retombait
    /// sur `Success` par défaut : les préfixes réellement produits par `AppError`
    /// (« Validation : », « Base de données : », « Sérialisation : »…) n'en contenant aucun,
    /// les échecs s'affichaient en vert avec une icône de coche.
    #[must_use]
    pub const fn from_error(error: &crate::shared::error::AppError) -> Self {
        match error {
            // Une annulation est une décision de l'utilisateur, pas une panne.
            crate::shared::error::AppError::Cancelled => Self::Warning,
            _ => Self::Error,
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
        let background = crate::ui::theme::styles::mix_panel(palette.panel, palette.canvas, 0.94);
        container::Style {
            background: Some(Background::Color(background)),
            text_color: Some(palette.text),
            border: Border {
                color: Color {
                    a: 0.70,
                    ..palette.border
                },
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
    use crate::shared::error::AppError;
    use crate::ui::theme::Tone;

    /// Les sept variantes réellement produites par le code, et non des chaînes fabriquées
    /// pour l'occasion. L'ancienne version de ce test devinait la nature par mots-clés
    /// français et passait au vert alors que « Sérialisation : missing field 'first_name' »
    /// s'affichait à l'écran avec l'icône de succès : aucun des huit mots recherchés ne
    /// figure dans les préfixes d'`AppError`.
    fn toutes_les_variantes() -> [AppError; 7] {
        [
            AppError::Validation("le poste est obligatoire".into()),
            AppError::NotFound("candidature".into()),
            AppError::Database("file is not a database".into()),
            AppError::Http("délai réseau dépassé".into()),
            AppError::Serialization("missing field `first_name` at line 1 column 344".into()),
            AppError::Provider("quota dépassé".into()),
            AppError::Cancelled,
        ]
    }

    #[test]
    fn aucune_erreur_ne_s_affiche_jamais_comme_un_succes() {
        for erreur in toutes_les_variantes() {
            let kind = Kind::from_error(&erreur);
            assert_ne!(
                kind,
                Kind::Success,
                "{erreur:?} s'afficherait en vert avec une icône de coche"
            );
            assert_ne!(
                kind,
                Kind::Info,
                "{erreur:?} passerait pour une information"
            );
        }
    }

    #[test]
    fn une_annulation_est_un_avertissement_pas_une_erreur() {
        assert_eq!(Kind::from_error(&AppError::Cancelled), Kind::Warning);
    }

    #[test]
    fn un_echec_reel_est_une_erreur() {
        assert_eq!(
            Kind::from_error(&AppError::Database("file is not a database".into())),
            Kind::Error
        );
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
}
