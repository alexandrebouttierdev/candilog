//! Atelier d'analyse d'offre et recommandations ATS.

use crate::app::state::RecommendationStatus;
use crate::app::{App, Message};
use crate::modules::ia::components::{recommendation, skill_list};
use crate::modules::ia::cv_model::{MatchScore, OfferAnalysis};
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon};
use crate::ui::components::score_gauge::tone_pour_score;
use crate::ui::components::{badge, field, layout, meter, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::{radius, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Background, Border, Element, Length, Theme};

use super::workflow::detected_company;

/// Atelier d'analyse : offre à coller, puis correspondance et suggestions.
pub(super) fn workbench(app: &App) -> Element<'_, Message> {
    let panel = match &app.offer_analysis {
        None => offer_panel(app),
        Some(analysis) => analysis_panel(app, analysis),
    };
    surface::scroll(container(panel).padding(space::XL))
        .height(Length::Fill)
        .into()
}

/// Étape 1 : carte info d'aide à gauche, offre à analyser à droite (grille 0.72fr/1.28fr).
fn offer_panel(app: &App) -> Element<'_, Message> {
    row![
        container(info_panel(app))
            .width(Length::FillPortion(72))
            .padding(space::XL)
            .style(primary_info_style),
        container(offer_content(app))
            .width(Length::FillPortion(128))
            .padding(space::XL)
            .style(styles::glass_card),
    ]
    .spacing(space::LG)
    .into()
}

/// Carte info de l'étape 1 : ce que l'analyse produira, et contexte de l'offre.
fn info_panel(app: &App) -> Element<'_, Message> {
    let mut content =
        column![
        surface::section_header("Analyse de l'offre", badge::badge("3 étapes", Tone::Neutral)),
        surface::divider(),
        info_row(
            Icon::Target,
            "Compétences",
            "Les compétences de l'offre sont comparées à votre profil, présentes ou manquantes.",
        ),
        info_row(
            Icon::Chart,
            "Score ATS",
            "Une jauge 0–100 mesure la correspondance entre l'offre et votre profil.",
        ),
        info_row(
            Icon::Sparkles,
            "Suggestions",
            "Des recommandations ciblées préparent l'amélioration puis l'export du CV.",
        ),
    ]
        .spacing(space::LG);

    if let Some(company) = detected_company(&app.offer_editor.text(), &app.data.entreprises) {
        content = content.push(surface::divider()).push(
            row![
                icon::icon(Icon::Building, icon::MD, icon::Ink::Toned(Tone::Accent)),
                column![typo::label("Entreprise détectée"), typo::body(company),]
                    .spacing(1)
                    .align_x(Alignment::Start),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Start),
        );
    }

    content.into()
}

/// Ligne d'information d'une carte : icône teintée, intitulé et description.
fn info_row<'a>(glyph: Icon, title: &'a str, detail: &'a str) -> Element<'a, Message> {
    row![
        icon::icon(glyph, icon::MD, icon::Ink::Toned(Tone::Accent)),
        column![typo::label(title), typo::caption(detail),]
            .spacing(1)
            .align_x(Alignment::Start),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Start)
    .into()
}

/// Carte offre de l'étape 1 : en-tête, éditeur et pied d'action.
fn offer_content(app: &App) -> Element<'_, Message> {
    let footer: Element<'_, Message> = if app.ai_is_running {
        state::running(
            "Analyse de l'offre",
            app.ai_elapsed_seconds,
            Message::CancelAi,
        )
    } else {
        container(
            row![
                typo::text_mono(
                    // `Content::text()` renvoie un saut de ligne terminal : sans le retirer,
                    // un éditeur vide affichait « 1 caractères » — faux compte et accord
                    // fautif. `format::plural` fait l'accord, comme le fait déjà l'écran
                    // voisin de la lettre de motivation.
                    format::plural(
                        app.offer_editor
                            .text()
                            .trim_end_matches('\n')
                            .chars()
                            .count(),
                        "caractère",
                        "caractères",
                    ),
                    font::MICRO,
                    font::MONO_REGULAR,
                ),
                layout::spacer(),
                controls::primary("Analyser l'offre", Some(Icon::Target))
                    .on_press(Message::AnalyzeOffer),
            ]
            .spacing(space::SM)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .into()
    };
    column![
        surface::section_header(
            "Offre ciblée",
            typo::caption("Collez le texte complet de l'annonce"),
        ),
        surface::divider(),
        row![
            typo::label("Texte intégral de l'offre"),
            layout::spacer(),
            controls::secondary("Coller depuis le presse-papiers", Some(Icon::Copy))
                .on_press(Message::PasteOfferFromClipboard),
        ]
        .align_y(Alignment::Center),
        field::editor(&app.offer_editor, "Collez ici le texte complet de l'offre…")
            .on_action(Message::OfferEditorAction)
            // Zone volontairement haute : une offre longue reste lisible sans
            // donner l'impression d'être tronquée. L'éditeur conserve son
            // défilement interne au-delà de cette hauteur.
            .height(Length::Fixed(480.0)),
        footer,
    ]
    .spacing(space::LG)
    .into()
}

/// Carte info de l'étape 1 : fond d'accent discret et filet doux
/// (`bg-primary/3.5`, `border-primary/15`), rayon de carte standard.
fn primary_info_style(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(alpha(palette.accent, 0.035))),
        text_color: Some(palette.text),
        border: Border {
            color: alpha(palette.accent, 0.15),
            width: stroke::HAIRLINE,
            radius: radius::CARD.into(),
        },
        ..container::Style::default()
    }
}

/// Étape 2 : correspondance profil × offre, suggestions ATS et version.
fn analysis_panel<'a>(app: &'a App, analysis: &'a OfferAnalysis) -> Element<'a, Message> {
    let present = present_skills(&analysis.score);
    let missing = missing_skills(&analysis.score);

    let mut content = column![
        surface::section_header(
            "Correspondance",
            badge::badge(
                format!("{} / 100", analysis.score.total),
                tone_pour_score(analysis.score.total),
            ),
        ),
        surface::divider(),
        skill_list("Présentes dans votre profil", &present, Tone::Success),
        skill_list("Manquantes dans votre profil", &missing, Tone::Neutral),
    ]
    .spacing(space::LG);

    match panel_footer_state(
        app.offer_analysis.is_some(),
        app.ai_is_running,
        app.cv_generation.is_some(),
    ) {
        PanelFooterState::ProposeGeneration => {
            content = content
                .push(surface::divider())
                .push(
                    controls::primary("Améliorer le CV", Some(Icon::Sparkles))
                        .on_press(Message::GenerateCv)
                        .width(Length::Fill),
                )
                .push(
                    container(typo::caption(
                        "Vous pourrez accepter ou refuser chaque suggestion.",
                    ))
                    .width(Length::Fill)
                    .center_x(Length::Fill),
                );
        }
        PanelFooterState::Generating => {
            content = content.push(surface::divider()).push(state::running(
                "Génération du CV…",
                app.ai_elapsed_seconds,
                Message::CancelAi,
            ));
        }
        PanelFooterState::None => {}
    }

    if let Some(generation) = &app.cv_generation {
        content = content
            .push(surface::divider())
            .push(surface::section_header(
                "Suggestions ATS",
                badge::count(generation.analysis.recommandations.len()),
            ))
            .push(meter::ats(generation.analysis.score));
        if generation.analysis.recommandations.is_empty() {
            content = content.push(state::empty_slot("Aucune suggestion à appliquer."));
        }
        for (index, item) in generation.analysis.recommandations.iter().enumerate() {
            let status = app
                .recommendation_states
                .get(index)
                .copied()
                .unwrap_or(RecommendationStatus::Pending);
            content = content.push(recommendation(
                item.section.clone(),
                item.impact,
                item.texte_original.clone(),
                item.texte_propose.clone(),
                status,
                Message::AcceptRecommendation(index),
                Message::RejectRecommendation(index),
            ));
        }
        content = content
            .push(surface::divider())
            .push(surface::section_header(
                "Enregistrer la version",
                controls::secondary("Enregistrer", Some(Icon::Save))
                    .on_press(Message::SaveGeneratedCv),
            ))
            .push(
                field::input("Nom de la version", &app.cv_version_name)
                    .on_input(Message::CvVersionNameChanged)
                    .width(Length::Fill),
            );
    }

    container(content)
        .padding(space::XL)
        .style(styles::glass_card)
        .into()
}

/// Action offerte en pied du panneau d'analyse, entre analyse et génération.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PanelFooterState {
    /// Rien à proposer : pas d'analyse, ou une génération est déjà affichée.
    None,
    /// Analyse terminée sans génération : proposer « Améliorer le CV ».
    ProposeGeneration,
    /// Génération en cours : montrer la progression et l'arrêt.
    Generating,
}

/// Décide du pied du panneau d'analyse.
///
/// Une génération déjà affichée prime : pendant une régénération, les suggestions
/// restent visibles et le pied ne reprend pas la main.
#[must_use]
pub(super) fn panel_footer_state(
    analysed: bool,
    running: bool,
    generated: bool,
) -> PanelFooterState {
    if !analysed || generated {
        PanelFooterState::None
    } else if running {
        PanelFooterState::Generating
    } else {
        PanelFooterState::ProposeGeneration
    }
}

/// Compétences de l'offre présentes dans le profil, prêtes à afficher.
///
/// L'analyse fournit déjà les listes `matched`/`missing` : on ne recalcule pas
/// le score, on nettoie seulement les doublons (casse ignorée, ordre préservé).
pub(super) fn present_skills(score: &MatchScore) -> Vec<String> {
    clean_skills(&score.matched)
}

/// Compétences de l'offre absentes du profil, prêtes à afficher.
pub(super) fn missing_skills(score: &MatchScore) -> Vec<String> {
    clean_skills(&score.missing)
}

/// Déduplication insensible à la casse : premier exemplaire conservé, ordre préservé.
fn clean_skills(skills: &[String]) -> Vec<String> {
    let mut seen = Vec::new();
    let mut cleaned = Vec::new();
    for skill in skills {
        let key = skill.to_lowercase();
        if !seen.contains(&key) {
            seen.push(key);
            cleaned.push(skill.clone());
        }
    }
    cleaned
}
