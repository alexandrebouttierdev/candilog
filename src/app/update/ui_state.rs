//! Transitions d'état du domaine `ui_state`.

use super::*;

pub(super) fn handles(message: &Message) -> bool {
    matches!(
        message,
        Message::AcceptRecommendation(..)
            | Message::RejectRecommendation(..)
            | Message::SelectCandidate(..)
            | Message::SelectCompany(..)
            | Message::SelectContact(..)
            | Message::SelectCvVersion(..)
            | Message::SelectLetter(..)
            | Message::ToggleFilters
            | Message::SortCandidates(..)
            | Message::StatisticsTabChanged(..)
            | Message::AtsPagePrev
            | Message::AtsPageNext
            | Message::LlmPagePrev
            | Message::LlmPageNext
            | Message::DocumentWidthChanged(..)
            | Message::FocusSearch
    )
}

pub(super) fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::AcceptRecommendation(index) => {
            let recommendation = app
                .cv_generation
                .as_ref()
                .and_then(|generation| generation.analysis.recommandations.get(index))
                .cloned();
            if let (Some(generation), Some(recommendation)) =
                (app.cv_generation.as_mut(), recommendation)
            {
                apply_recommendation(&mut generation.cv, &recommendation);
                if let Some(status) = app.recommendation_states.get_mut(index) {
                    *status = super::super::state::RecommendationStatus::Accepted;
                }
            }
        }
        Message::RejectRecommendation(index) => {
            if let Some(status) = app.recommendation_states.get_mut(index) {
                *status = super::super::state::RecommendationStatus::Rejected;
            }
        }
        Message::SelectCandidate(id) => app.selected_candidate = id,
        Message::SelectCompany(id) => app.selected_company = id,
        Message::SelectContact(id) => app.selected_contact = id,
        Message::SelectCvVersion(id) => app.selected_cv = id,
        Message::SelectLetter(id) => app.selected_letter = id,
        Message::ToggleFilters => app.filters_open = !app.filters_open,
        Message::SortCandidates(column) => {
            if let Some(sort) = super::super::state::CandidateSort::from_column(column) {
                if sort == app.candidate_sort {
                    app.candidate_sort_descending = !app.candidate_sort_descending;
                } else {
                    app.candidate_sort = sort;
                    app.candidate_sort_descending = false;
                }
                app.candidate_page = 1;
                return recharger(app);
            }
        }
        Message::StatisticsTabChanged(tab) => app.statistics_tab = tab,
        // Les compteurs sont 1-based et bornés **ici**, dans la transition d'état, et non
        // seulement à l'affichage : borner en aval laissait le compteur dériver sans que rien
        // ne bouge à l'écran, puis exiger autant de clics en sens inverse pour revenir dans
        // la plage utile.
        Message::AtsPagePrev => {
            app.ats_page = app.ats_page.saturating_sub(1).max(1);
            return recharger(app);
        }
        Message::AtsPageNext => {
            app.ats_page = app.ats_page.saturating_add(1).min(app.ats_total_pages());
            return recharger(app);
        }
        Message::LlmPagePrev => {
            app.llm_page = app.llm_page.saturating_sub(1).max(1);
            return recharger(app);
        }
        Message::LlmPageNext => {
            app.llm_page = app.llm_page.saturating_add(1).min(app.llm_total_pages());
            return recharger(app);
        }
        Message::DocumentWidthChanged(width) => app.document_width = width,
        Message::FocusSearch => {
            return iced::widget::text_input::focus(SEARCH_FIELD_ID);
        }
        _ => unreachable!("message routé vers un domaine incorrect"),
    }
    Task::none()
}
