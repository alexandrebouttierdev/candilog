//! Rendu des objets IA : suggestions, recommandations, état des opérations.

use crate::app::state::RecommendationStatus;
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon};
use crate::ui::components::{badge, layout, typo};
use crate::ui::theme::metrics::{radius, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Ton et libellé associés à la décision prise sur une recommandation.
#[must_use]
pub const fn recommendation_state(status: RecommendationStatus) -> (&'static str, Tone) {
    match status {
        RecommendationStatus::Pending => ("En attente", Tone::Neutral),
        RecommendationStatus::Accepted => ("Appliquée", Tone::Success),
        RecommendationStatus::Rejected => ("Refusée", Tone::Danger),
    }
}

/// Recommandation ATS présentée comme une carte : section, gain, original
/// estompé, texte proposé sur fond teinté succès et décision à prendre.
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
            badge::badge(format!("+{impact} pts ATS"), Tone::Success),
            layout::spacer(),
            badge::badge(label, tone),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
        typo::body(original).style(styles::muted_text),
        proposed_box(proposed),
    ]
    .spacing(space::SM)
    .width(Length::Fill);

    if status == RecommendationStatus::Pending {
        body = body.push(
            row![
                layout::spacer(),
                controls::secondary("Refuser", Some(Icon::Close)).on_press(on_reject),
                controls::primary("Accepter", Some(Icon::Check)).on_press(on_accept),
            ]
            .spacing(space::SM)
            .align_y(Alignment::Center),
        );
    }

    container(body)
        .padding(space::LG)
        .width(Length::Fill)
        .style(styles::glass_card)
        .into()
}

/// Texte proposé dans une encoche teintée succès, filet vertical à gauche.
fn proposed_box<'a, Message: 'a>(proposed: String) -> Element<'a, Message> {
    row![
        container(Space::new(stroke::MARKER, Length::Fill)).style(|theme: &Theme| {
            container::Style {
                background: Some(Background::Color(Tone::Success.color(&tokens(theme)))),
                border: Border {
                    radius: radius::MARKER.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }
        }),
        container(typo::body(proposed))
            .padding([space::SM, space::MD])
            .width(Length::Fill)
            .style(|theme: &Theme| {
                let palette = tokens(theme);
                container::Style {
                    background: Some(Background::Color(Tone::Success.surface(&palette))),
                    border: Border {
                        color: Tone::Success.edge(&palette),
                        width: stroke::HAIRLINE,
                        radius: radius::CONTROL.into(),
                    },
                    ..container::Style::default()
                }
            }),
    ]
    .spacing(space::SM)
    .into()
}

/// Liste de compétences détectées, sous forme de pastilles, avec compteur.
pub fn skill_list<'a, Message: 'a>(
    title: &'a str,
    skills: &[String],
    tone: Tone,
) -> Element<'a, Message> {
    let chips: Element<'a, Message> = if skills.is_empty() {
        typo::caption("Aucune").into()
    } else {
        let mut line = row![].spacing(space::XS);
        for skill in skills {
            line = line.push(badge::badge(skill.as_str(), tone));
        }
        line.wrap().into()
    };
    column![
        row![
            typo::label(title),
            iced::widget::Space::with_width(Length::Fill),
            badge::count_toned(skills.len(), tone),
        ]
        .align_y(Alignment::Center),
        chips,
    ]
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
