//! En-tête et progression du parcours de génération de CV.

use crate::app::{App, Message};
use crate::modules::entreprises::model::Entreprise;
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon};
use crate::ui::components::score_gauge::{gauge, score_label};
use crate::ui::components::workflow::{steps, StepState, WorkflowStep};
use crate::ui::components::{badge, layout, typo};
use crate::ui::theme::metrics::{size, space};
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, Stack};
use iced::{Alignment, Element, Length};

/// En-tête : badge « IA », entreprise détectée puis actions révélées par l'avancée.
pub(super) fn actions(app: &App) -> Element<'_, Message> {
    let mut actions = row![badge::badge("IA", Tone::Accent)]
        .spacing(space::SM)
        .align_y(Alignment::Center);
    if let Some(company) = detected_company(&app.offer_editor.text(), &app.data.entreprises) {
        actions = actions.push(company_pill(company));
    }
    if app.ai_is_running {
        actions = actions
            .push(controls::ghost("Tout annuler", Some(Icon::Stop)).on_press(Message::CancelAi));
    }
    if app.offer_analysis.is_some() {
        actions = actions.push(
            controls::secondary("Régénérer", Some(Icon::Refresh)).on_press(Message::GenerateCv),
        );
    }
    if app.cv_generation.is_some() {
        actions = actions.push(
            controls::primary("Exporter PDF", Some(Icon::Download))
                .on_press(Message::ExportGeneratedCvPdf),
        );
    }
    actions.into()
}

/// Pastille « entreprise détectée » : bâtiment et nom, en ton neutre.
fn company_pill<'a, Message: 'a>(company: String) -> Element<'a, Message> {
    container(
        row![
            icon::icon(Icon::Building, icon::SM, icon::Ink::Muted),
            typo::caption(company),
        ]
        .spacing(space::XS)
        .align_y(Alignment::Center),
    )
    .height(size::TAG)
    .padding([space::XXS, space::SM])
    .align_y(Alignment::Center)
    .style(styles::toned(Tone::Neutral))
    .into()
}

/// Détecte une entreprise connue citée dans le texte de l'offre.
pub(super) fn detected_company(offer_text: &str, companies: &[Entreprise]) -> Option<String> {
    let needle = offer_text.to_lowercase();
    companies
        .iter()
        .map(|company| company.nom.trim())
        .filter(|nom| !nom.is_empty())
        // Les noms courts (ex. « IT », « Dev » ou « SA ») ne doivent pas être
        // reconnus au milieu d'un autre mot. Les noms composés et précis sont
        // testés en premier pour éviter qu'un sous-nom masque la bonne société.
        .filter(|nom| nom.chars().count() >= 3)
        .filter_map(|nom| {
            whole_phrase_position(&needle, &nom.to_lowercase()).and_then(|position| {
                company_context_score(&needle, &nom.to_lowercase(), position)
                    .map(|score| (score, position, nom))
            })
        })
        .max_by_key(|(score, position, nom)| {
            (*score, std::cmp::Reverse(*position), nom.chars().count())
        })
        .map(|(_, _, nom)| nom.to_owned())
}

fn whole_phrase_position(text: &str, phrase: &str) -> Option<usize> {
    text.match_indices(phrase).find_map(|(start, _)| {
        let end = start + phrase.len();
        let before = text[..start].chars().next_back();
        let after = text[end..].chars().next();
        (!before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric))
            .then_some(start)
    })
}

/// Évite de prendre une entreprise citée comme exemple, client ou concurrent.
/// Une détection est retenue uniquement si le nom est dans un contexte explicite
/// (`chez`, `rejoignez`, `notre client`...) ou s'il forme un en-tête proche du début.
fn company_context_score(text: &str, phrase: &str, position: usize) -> Option<u8> {
    let before_start = position.saturating_sub(96);
    let before = format!(" {}", &text[before_start..position]);
    let line_start = text[..position].rfind('\n').map_or(0, |index| index + 1);
    let line_end = text[position..]
        .find('\n')
        .map_or(text.len(), |index| position + index);
    let line = text[line_start..line_end].trim();
    let explicit_markers = [
        " chez ",
        " rejoignez ",
        " au sein de ",
        " notre client ",
        " l'entreprise ",
        " pour ",
        " entreprise ",
    ];
    if explicit_markers
        .iter()
        .any(|marker| before.contains(marker))
    {
        return Some(3);
    }
    if position < 400 && line.to_lowercase().starts_with(phrase) && line.chars().count() <= 120 {
        return Some(2);
    }
    None
}

/// Bandeau sous l'en-tête : étapes du parcours à gauche, jauge ATS à droite.
pub(super) fn overview(app: &App) -> Element<'_, Message> {
    let analysed = app.offer_analysis.is_some();
    let generated = app.cv_generation.is_some();
    let items = [
        WorkflowStep::new(
            "Analyser",
            "L'offre",
            if analysed {
                StepState::Done
            } else {
                StepState::Active
            },
        ),
        WorkflowStep::new(
            "Améliorer",
            "Le CV",
            if generated {
                StepState::Done
            } else if analysed {
                StepState::Active
            } else {
                StepState::Pending
            },
        ),
        WorkflowStep::new(
            "Exporter",
            "Le PDF",
            if generated {
                StepState::Active
            } else {
                StepState::Pending
            },
        ),
    ];
    let score = app
        .offer_analysis
        .as_ref()
        .map_or(0, |analysis| analysis.score.total);

    row![
        steps(&items),
        layout::spacer(),
        column![
            Stack::with_children(vec![
                gauge(score, 48.0),
                container(score_label(score))
                    .center_x(Length::Fixed(48.0))
                    .center_y(Length::Fixed(48.0))
                    .into(),
            ]),
            typo::caption("Score ATS"),
        ]
        .spacing(space::XS)
        .align_x(Alignment::Center),
    ]
    .spacing(space::LG)
    .align_y(Alignment::Center)
    .into()
}
