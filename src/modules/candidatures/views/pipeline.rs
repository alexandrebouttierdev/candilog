//! Pipeline Kanban : colonnes verre, en-tête pastille + compteur, zone de dépôt marquée.

use crate::app::{App, Message};
use crate::modules::candidatures::components::{column_label, kanban_card, status_tone, PIPELINE};
use crate::modules::candidatures::model::Candidature;
use crate::ui::components::{badge, state, surface, typo};
use crate::ui::theme::metrics::{size, space};
use crate::ui::theme::styles;
use crate::ui::theme::typography::SEMIBOLD;
use crate::ui::theme::{Marker, Tone};
use iced::widget::{column, container, mouse_area, responsive, row, Space};
use iced::{Alignment, Element, Length};

/// Largeur d'une colonne : elle occupe l'espace disponible sur un grand écran
/// et retombe sur sa largeur minimale, avec défilement, sur une petite fenêtre.
fn column_width(available: f32) -> f32 {
    let gaps = space::MD * (PIPELINE.len() - 1) as f32;
    let share = (available - gaps) / PIPELINE.len() as f32;
    share.max(size::KANBAN_COLUMN)
}

/// Rend le pipeline complet à partir des candidatures déjà filtrées.
pub fn view<'a>(app: &'a App, candidates: &[&'a Candidature]) -> Element<'a, Message> {
    let groups: Vec<Vec<&'a Candidature>> = PIPELINE
        .iter()
        .map(|status| {
            candidates
                .iter()
                .copied()
                .filter(|candidate| candidate.statut == *status)
                .collect()
        })
        .collect();

    responsive(move |viewport| {
        let width = column_width(viewport.width);
        let mut board = row![].spacing(space::MD).height(Length::Fill);
        for (index, status) in PIPELINE.into_iter().enumerate() {
            let is_target =
                app.dragging_candidate.is_some() && app.drag_target_status == Some(status);
            board = board.push(
                mouse_area(pipeline_column(
                    app,
                    status,
                    &groups[index],
                    is_target,
                    width,
                ))
                .on_enter(Message::CandidateDragHovered(status))
                .on_release(Message::CandidateDropped(status)),
            );
        }
        mouse_area(surface::scroll_x(container(board).height(Length::Fill)).height(Length::Fill))
            .on_release(Message::CandidateDragCancelled)
            .into()
    })
    .into()
}

fn pipeline_column<'a>(
    app: &'a App,
    status: crate::modules::candidatures::model::StatutCandidature,
    candidates: &[&'a Candidature],
    is_target: bool,
    width: f32,
) -> Element<'a, Message> {
    let tone = status_tone(status);
    let header = container(
        row![
            badge::marker(tone, Marker::Solid),
            typo::body(column_label(status)).font(SEMIBOLD),
            badge::count(candidates.len()),
            Space::with_width(Length::Fill),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .height(38.0)
    .padding([0.0, space::MD])
    .align_y(Alignment::Center);

    let body: Element<'a, Message> = if candidates.is_empty() {
        state::empty_slot(if is_target {
            "Relâchez pour déplacer ici"
        } else {
            "Aucune candidature"
        })
    } else {
        let mut cards = column![].spacing(space::SM).padding(space::MD);
        for candidate in candidates {
            cards = cards.push(kanban_card(
                candidate,
                app.selected_candidate == Some(candidate.id),
                app.hovered_card == Some(candidate.id),
                Message::CandidatePressed(candidate.id),
                Message::CandidateMoved,
                Message::CandidateReleased,
                Message::CandidateCardHovered(candidate.id),
                Message::CandidateCardExited,
            ));
        }
        surface::scroll(cards).height(Length::Fill).into()
    };

    let drop_zone = container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(styles::drop_zone(is_target));

    let drop_hint: Element<'a, Message> = if is_target && !candidates.is_empty() {
        container(typo::meta_toned("Relâchez pour déplacer ici", Tone::Accent))
            .padding([space::SM, space::LG])
            .width(Length::Fill)
            .into()
    } else {
        Space::with_height(0).into()
    };

    container(column![header, surface::divider(), drop_zone, drop_hint].height(Length::Fill))
        .width(width)
        .height(Length::Fill)
        .style(styles::kanban_column)
        .into()
}

#[cfg(test)]
mod tests {
    use super::column_width;
    use crate::ui::theme::metrics::{size, space};

    #[test]
    fn les_colonnes_occupent_un_grand_ecran() {
        // 1800 px de fenêtre, moins la barre latérale et les marges.
        let width = column_width(1560.0);
        assert!(
            width > size::KANBAN_COLUMN,
            "l'espace libre doit être utilisé"
        );
        let total = 4.0f32.mul_add(width, 3.0 * space::MD);
        assert!(
            (total - 1560.0).abs() < 1.0,
            "le pipeline doit remplir la largeur"
        );
    }

    #[test]
    fn les_colonnes_gardent_leur_lisibilite_en_petite_fenetre() {
        let width = column_width(820.0);
        assert!((width - size::KANBAN_COLUMN).abs() < f32::EPSILON);
    }

    #[test]
    fn la_largeur_ne_devient_jamais_negative() {
        assert!(column_width(0.0) >= size::KANBAN_COLUMN);
    }
}
