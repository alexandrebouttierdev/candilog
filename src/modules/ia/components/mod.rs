//! Rendu des objets IA : suggestions, recommandations, état des opérations.

use crate::app::state::RecommendationStatus;
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon};
use crate::ui::components::{badge, layout, surface, typo};
use crate::ui::theme::metrics::space;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Ton et libellé associés à la décision prise sur une recommandation.
#[must_use]
pub const fn recommendation_state(status: RecommendationStatus) -> (&'static str, Tone) {
    match status {
        RecommendationStatus::Pending => ("En attente", Tone::Neutral),
        RecommendationStatus::Accepted => ("Appliquée", Tone::Success),
        RecommendationStatus::Rejected => ("Refusée", Tone::Danger),
    }
}

/// Recommandation ATS présentée comme une entrée de liste, pas comme une carte.
#[allow(clippy::too_many_arguments)]
pub fn recommendation<'a, Message: Clone + 'a>(
    section: String,
    impact: u8,
    original: String,
    proposed: String,
    status: RecommendationStatus,
    on_accept: Message,
    on_reject: Message,
) -> Element<'a, Message> {
    let (label, tone) = recommendation_state(status);
    let mut body = column![
        row![
            icon::toned(Icon::Sparkles, Tone::Accent),
            typo::item_strong(section),
            badge::badge(format!("+{impact} pts"), Tone::Success),
            layout::spacer(),
            badge::badge(label, tone),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
        typo::caption(original),
        typo::body(proposed),
    ]
    .spacing(space::XS)
    .width(Length::Fill);

    if status == RecommendationStatus::Pending {
        body = body.push(
            row![
                layout::spacer(),
                controls::ghost("Refuser", Some(Icon::Close)).on_press(on_reject),
                controls::secondary("Appliquer", Some(Icon::Check)).on_press(on_accept),
            ]
            .spacing(space::SM)
            .align_y(Alignment::Center),
        );
    }

    column![
        container(body)
            .padding([space::LG, 0.0])
            .width(Length::Fill),
        surface::divider(),
    ]
    .into()
}

/// Liste de compétences détectées, présentes ou manquantes.
pub fn skill_list<'a, Message: 'a>(
    title: &'a str,
    skills: &[String],
    tone: Tone,
) -> Element<'a, Message> {
    let body: Element<'a, Message> = if skills.is_empty() {
        typo::caption("Aucune").into()
    } else {
        typo::toned(skills.join(" · "), tone).into()
    };
    column![typo::label(title), body]
        .spacing(space::XS)
        .width(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::recommendation_state;
    use crate::app::state::RecommendationStatus;
    use crate::ui::theme::Tone;

    #[test]
    fn chaque_decision_porte_un_libelle_et_un_ton_propres() {
        let states = [
            RecommendationStatus::Pending,
            RecommendationStatus::Accepted,
            RecommendationStatus::Rejected,
        ];
        for (index, status) in states.iter().enumerate() {
            for other in &states[index + 1..] {
                assert_ne!(
                    recommendation_state(*status).0,
                    recommendation_state(*other).0
                );
                assert_ne!(
                    recommendation_state(*status).1,
                    recommendation_state(*other).1
                );
            }
        }
    }

    #[test]
    fn une_decision_en_attente_reste_neutre() {
        assert_eq!(
            recommendation_state(RecommendationStatus::Pending).1,
            Tone::Neutral
        );
    }
}
