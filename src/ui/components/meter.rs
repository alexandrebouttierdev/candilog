//! Indicateurs chiffrés : métriques, jauges, barres comparées.
//!
//! Chaque indicateur se lit sans couleur : la valeur et son libellé suffisent.
//! Aucun graphique n'est ajouté pour remplir l'espace.

use super::badge;
use super::typo;
use crate::ui::theme::metrics::{radius, space};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::Tone;
use iced::widget::{column, container, progress_bar, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Niveau qualitatif d'un score ATS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtsLevel {
    /// Score à retravailler.
    Weak,
    /// Score exploitable mais perfectible.
    Fair,
    /// Score solide.
    Strong,
}

impl AtsLevel {
    /// Libellé accessible, indépendant de la couleur.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Weak => "À renforcer",
            Self::Fair => "Correct",
            Self::Strong => "Solide",
        }
    }

    /// Ton sémantique du niveau.
    #[must_use]
    pub const fn tone(self) -> Tone {
        match self {
            Self::Weak => Tone::Danger,
            Self::Fair => Tone::Warning,
            Self::Strong => Tone::Success,
        }
    }
}

/// Classe un score ATS borné sur 100.
#[must_use]
pub const fn ats_level(score: u8) -> AtsLevel {
    if score < 50 {
        AtsLevel::Weak
    } else if score < 75 {
        AtsLevel::Fair
    } else {
        AtsLevel::Strong
    }
}

/// Affichage canonique d'un score ATS : valeur, niveau écrit, progression.
pub fn ats<'a, Message: 'a>(score: u8) -> Element<'a, Message> {
    let bounded = score.min(100);
    let level = ats_level(bounded);
    column![
        row![
            typo::label("Score ATS"),
            Space::with_width(Length::Fill),
            typo::item_strong(format!("{bounded} / 100")),
            badge::badge(level.label(), level.tone()),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
        progress_bar(0.0..=100.0, f32::from(bounded))
            .height(4)
            .style(styles::progress(level.tone())),
    ]
    .spacing(space::SM)
    .into()
}

/// Version compacte du score ATS, pour une ligne de liste ou une carte.
pub fn ats_inline<'a, Message: 'a>(score: u8) -> Element<'a, Message> {
    let bounded = score.min(100);
    let level = ats_level(bounded);
    badge::badge(format!("ATS {bounded}"), level.tone())
}

/// Indicateur d'une bande de synthèse.
pub fn metric<'a, Message: 'a>(
    label: &'a str,
    value: impl Into<String>,
    tone: Tone,
) -> Element<'a, Message> {
    column![
        typo::metric(value.into()).style(styles::toned_text(tone)),
        typo::caption(label),
    ]
    .spacing(2)
    .into()
}

/// Indicateur accompagné d'une précision, pour guider vers une action.
pub fn metric_with_hint<'a, Message: 'a>(
    label: &'a str,
    value: impl Into<String>,
    hint: impl Into<String>,
    tone: Tone,
) -> Element<'a, Message> {
    column![
        typo::metric(value.into()).style(styles::toned_text(tone)),
        typo::caption(label),
        typo::meta_toned(hint.into(), tone),
    ]
    .spacing(2)
    .into()
}

/// Barre d'un graphique de répartition, avec sa valeur écrite en clair.
pub fn bar<'a, Message: 'a>(
    label: String,
    value: usize,
    total: usize,
    tone: Tone,
) -> Element<'a, Message> {
    let ratio = if total == 0 {
        0.0
    } else {
        value as f32 / total as f32
    };
    let percent = (ratio * 100.0).round() as u32;
    column![
        row![
            typo::body(label),
            Space::with_width(Length::Fill),
            typo::meta(format!("{value}")),
            typo::caption(format!("{percent} %")),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
        track(ratio, tone),
    ]
    .spacing(space::XS)
    .into()
}

/// Rail dessiné d'une barre de répartition.
fn track<'a, Message: 'a>(ratio: f32, tone: Tone) -> Element<'a, Message> {
    let filled = (ratio.clamp(0.0, 1.0) * 1000.0).round() as u16;
    let empty = 1000_u16.saturating_sub(filled);
    let fill = container(Space::new(Length::Fill, 5.0))
        .width(Length::FillPortion(filled.max(1)))
        .style(move |theme: &Theme| container::Style {
            background: Some(Background::Color(tone.color(&tokens(theme)))),
            border: Border {
                radius: radius::PILL.into(),
                ..Border::default()
            },
            ..container::Style::default()
        });
    let rest = Space::with_width(Length::FillPortion(empty.max(1)));
    container(if filled == 0 {
        row![rest]
    } else if empty == 0 {
        row![fill]
    } else {
        row![fill, rest]
    })
    .height(5.0)
    .width(Length::Fill)
    .style(|theme: &Theme| container::Style {
        background: Some(Background::Color(tokens(theme).sunken)),
        border: Border {
            color: tokens(theme).border,
            width: 1.0,
            radius: radius::PILL.into(),
        },
        ..container::Style::default()
    })
    .into()
}

/// Étape d'un entonnoir : libellé, effectif, taux par rapport à l'étape amont.
pub fn funnel_step<'a, Message: 'a>(
    label: String,
    value: usize,
    reference: usize,
    tone: Tone,
) -> Element<'a, Message> {
    bar(label, value, reference, tone)
}

#[cfg(test)]
mod tests {
    use super::{ats_level, AtsLevel};
    use crate::ui::theme::Tone;

    #[test]
    fn les_seuils_ats_sont_couverts() {
        assert_eq!(ats_level(0), AtsLevel::Weak);
        assert_eq!(ats_level(49), AtsLevel::Weak);
        assert_eq!(ats_level(50), AtsLevel::Fair);
        assert_eq!(ats_level(74), AtsLevel::Fair);
        assert_eq!(ats_level(75), AtsLevel::Strong);
        assert_eq!(ats_level(100), AtsLevel::Strong);
    }

    #[test]
    fn chaque_niveau_ats_porte_un_libelle_lisible_sans_couleur() {
        for level in [AtsLevel::Weak, AtsLevel::Fair, AtsLevel::Strong] {
            assert!(!level.label().is_empty());
        }
        assert_ne!(AtsLevel::Weak.label(), AtsLevel::Fair.label());
        assert_ne!(AtsLevel::Fair.label(), AtsLevel::Strong.label());
    }

    #[test]
    fn chaque_niveau_ats_porte_un_ton_distinct() {
        assert_eq!(AtsLevel::Weak.tone(), Tone::Danger);
        assert_eq!(AtsLevel::Fair.tone(), Tone::Warning);
        assert_eq!(AtsLevel::Strong.tone(), Tone::Success);
    }

    #[test]
    fn un_score_hors_borne_reste_classe_comme_solide() {
        assert_eq!(ats_level(u8::MAX), AtsLevel::Strong);
    }
}
