//! Étapes numérotées d'un parcours en workflow (3 étapes max).

use super::typo;
use crate::ui::theme::metrics::{radius, space};
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

/// État d'une étape du workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    /// Étape franchie.
    Done,
    /// Étape en cours.
    Active,
    /// Étape à venir.
    Pending,
}

impl StepState {
    /// Les trois états, dans l'ordre du parcours.
    pub const ALL: [Self; 3] = [Self::Done, Self::Active, Self::Pending];
}

/// Étape d'un workflow.
#[derive(Debug, Clone)]
pub struct WorkflowStep {
    /// Libellé de l'étape.
    pub label: String,
    /// Détail affiché sous le libellé.
    pub detail: String,
    /// État courant.
    pub state: StepState,
}

impl WorkflowStep {
    /// Construit une étape.
    #[must_use]
    pub fn new(label: impl Into<String>, detail: impl Into<String>, state: StepState) -> Self {
        Self {
            label: label.into(),
            detail: detail.into(),
            state,
        }
    }
}

/// Barre de workflow à 3 étapes, numérotées et colorées par état.
pub fn steps<'a, Message: 'a>(steps: &'a [WorkflowStep]) -> Element<'a, Message> {
    let mut cells = row![].spacing(0);
    for (index, step) in steps.iter().enumerate() {
        let (tone, number) = match step.state {
            StepState::Done => (Tone::Success, "✓".to_string()),
            StepState::Active => (Tone::Accent, (index + 1).to_string()),
            StepState::Pending => (Tone::Neutral, (index + 1).to_string()),
        };
        let pastille = container(typo::text_mono(number, 11.0, font::MONO_SEMIBOLD))
            .width(28.0)
            .height(28.0)
            .center_x(Length::Fixed(28.0))
            .center_y(Length::Fixed(28.0))
            .style(move |theme: &Theme| {
                let palette = tokens(theme);
                container::Style {
                    background: Some(Background::Color(tone.surface(&palette))),
                    border: Border {
                        radius: radius::PILL.into(),
                        ..Border::default()
                    },
                    ..container::Style::default()
                }
            });
        let cell = container(
            row![
                pastille,
                column![
                    typo::body(step.label.as_str()).font(font::SEMIBOLD),
                    typo::caption(step.detail.as_str()),
                ]
                .spacing(0),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center),
        )
        .padding([space::MD, space::LG])
        .width(Length::FillPortion(1));
        cells = cells.push(cell);
    }
    container(cells)
        .max_width(760.0)
        .width(Length::Fill)
        .style(move |theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(Color {
                    a: 0.35,
                    ..palette.panel
                })),
                border: Border {
                    color: palette.border,
                    width: 1.0,
                    radius: radius::CONTROL.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::{StepState, WorkflowStep};

    #[test]
    fn les_etats_couvrent_le_parcours() {
        assert_eq!(StepState::ALL.len(), 3);
        let step = WorkflowStep::new("Analyser", "L'offre", StepState::Active);
        assert_eq!(step.label, "Analyser");
    }
}
