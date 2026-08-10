//! CV Generator : workflow, jauge de score et atelier d'analyse à gauche,
//! document A4 posé à droite.

use crate::app::state::RecommendationStatus;
use crate::app::{App, Message};
use crate::modules::ia::components::{recommendation, skill_list};
use crate::modules::ia::cv_model::{CvGeneration, MatchScore, OfferAnalysis};
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::Icon;
use crate::ui::components::score_gauge::{gauge, score_label, tone_pour_score};
use crate::ui::components::workflow::{steps, StepState, WorkflowStep};
use crate::ui::components::{badge, document, field, layout, meter, state, surface, typo};
use crate::ui::theme::metrics::{space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, Stack};
use iced::{Alignment, Border, Element, Length, Shadow, Theme, Vector};

/// Rend l'écran du générateur de CV.
pub fn view(app: &App) -> Element<'_, Message> {
    layout::screen(
        header::page_header(
            Icon::Sparkles,
            "CV Generator",
            "Offre → analyse → CV optimisé",
            actions(app),
        ),
        layout::workspace(
            column![
                overview(app),
                layout::columns([workbench(app), preview(app)]),
            ]
            .spacing(space::LG)
            .height(Length::Fill),
        ),
    )
}

/// Actions de l'en-tête, révélées par l'avancée du parcours.
fn actions(app: &App) -> Element<'_, Message> {
    let mut actions = row![].spacing(space::SM).align_y(Alignment::Center);
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

/// Bandeau sous l'en-tête : étapes du parcours à gauche, jauge ATS à droite.
fn overview(app: &App) -> Element<'_, Message> {
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

/// Atelier d'analyse : offre à coller, puis correspondance et suggestions.
fn workbench(app: &App) -> Element<'_, Message> {
    let panel = match &app.offer_analysis {
        None => offer_panel(app),
        Some(analysis) => analysis_panel(app, analysis),
    };
    surface::scroll(container(panel).padding(space::XL))
        .height(Length::Fill)
        .into()
}

/// Étape 1 : l'offre à analyser, avec compteur et action principale.
fn offer_panel(app: &App) -> Element<'_, Message> {
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
                    format!("{} caractères", app.offer_editor.text().chars().count()),
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
    let content = column![
        surface::section_header(
            "Offre ciblée",
            typo::caption("Collez le texte complet de l'annonce"),
        ),
        surface::divider(),
        field::editor(&app.offer_editor, "Collez ici le texte complet de l'offre…")
            .on_action(Message::OfferEditorAction)
            .height(Length::Fixed(245.0)),
        footer,
    ]
    .spacing(space::LG);
    container(content)
        .padding(space::XL)
        .style(styles::glass_card)
        .into()
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

/// Aperçu du document A4, posé sur son plan de travail.
fn preview(app: &App) -> Element<'_, Message> {
    let bar = document::workbench_bar(
        "Aperçu du document",
        row![typo::caption("A4"), zoom_controls(app),]
            .spacing(space::MD)
            .align_y(Alignment::Center),
    );

    let page: Element<'_, Message> = match &app.cv_generation {
        Some(generation) => page_content(app, generation, app.cv_version_name.clone()),
        None => placeholder(app),
    };

    container(
        column![
            container(bar).width(Length::Fill),
            surface::divider(),
            document::workspace(document::page(app.document_width, page)),
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|theme: &Theme| {
        let palette = tokens(theme);
        container::Style {
            border: Border {
                color: palette.border,
                width: stroke::HAIRLINE,
                ..Border::default()
            },
            shadow: Shadow {
                color: palette.shadow,
                offset: Vector::new(0.0, 10.0),
                blur_radius: 28.0,
            },
            ..container::Style::default()
        }
    })
    .into()
}

fn zoom_controls(app: &App) -> Element<'_, Message> {
    let mut segments = Vec::new();
    for width in document::zoom_widths() {
        segments.push(
            controls::segment(
                format!("{} %", document::zoom_percent(width)),
                (app.document_width - width).abs() < f32::EPSILON,
            )
            .on_press(Message::DocumentWidthChanged(width)),
        );
    }
    controls::segmented(segments)
}

/// Contenu de la page A4 pour une génération donnée.
pub fn page_content<'a>(
    app: &'a App,
    generation: &'a CvGeneration,
    _title: String,
) -> Element<'a, Message> {
    let profile = &app.data.profile.personal;
    let mut page = column![
        document::heading(format!("{} {}", profile.first_name, profile.last_name).trim()),
        document::subheading(crate::ui::format::or_else(
            profile.headline.as_deref(),
            "Titre professionnel"
        )),
        document::body_muted(contact_line(app)),
        iced::widget::Space::with_height(space::LG),
        document::rubric("Profil"),
        document::body(generation.cv.summary.clone()),
    ]
    .spacing(space::SM)
    .width(Length::Fill);

    if !generation.cv.skills.is_empty() {
        page = page
            .push(iced::widget::Space::with_height(space::SM))
            .push(document::rubric("Compétences"))
            .push(document::body(generation.cv.skills.join(" · ")));
    }

    if !generation.cv.experiences.is_empty() {
        page = page
            .push(iced::widget::Space::with_height(space::SM))
            .push(document::rubric("Expériences"));
        for experience in &generation.cv.experiences {
            page = page.push(document::entry(
                format!("{} — {}", experience.title, experience.company),
                String::new(),
                experience.description.clone(),
            ));
        }
    }

    if !generation.cv.education.is_empty() {
        page = page
            .push(iced::widget::Space::with_height(space::SM))
            .push(document::rubric("Formation"));
        for education in &generation.cv.education {
            page = page.push(document::entry(
                education.degree.clone(),
                String::new(),
                education.school.clone(),
            ));
        }
    }

    page.into()
}

fn placeholder(app: &App) -> Element<'_, Message> {
    let profile = &app.data.profile.personal;
    let name = format!("{} {}", profile.first_name, profile.last_name)
        .trim()
        .to_owned();
    column![
        document::heading(if name.is_empty() {
            "Votre nom".to_owned()
        } else {
            name
        }),
        document::subheading(crate::ui::format::or_else(
            profile.headline.as_deref(),
            "Titre professionnel"
        )),
        document::body_muted(contact_line(app)),
        iced::widget::Space::with_height(space::MAX),
        document::rubric("Profil"),
        document::body(crate::ui::format::or_else(
            profile.summary.as_deref(),
            "Analysez une offre pour générer un CV adapté à sa fiche de poste.",
        )),
        iced::widget::Space::with_height(space::MD),
        document::body_muted(format!(
            "{} · {}",
            crate::ui::format::plural(
                app.data.profile.experiences.len(),
                "expérience",
                "expériences"
            ),
            crate::ui::format::plural(app.data.profile.skills.len(), "compétence", "compétences"),
        )),
    ]
    .spacing(space::SM)
    .width(Length::Fill)
    .into()
}

fn contact_line(app: &App) -> String {
    let personal = &app.data.profile.personal;
    [
        Some(personal.email.as_str()).filter(|value| !value.is_empty()),
        personal.phone.as_deref(),
        personal.city.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter(|value| !value.trim().is_empty())
    .collect::<Vec<_>>()
    .join(" · ")
}

/// Compétences de l'offre présentes dans le profil, prêtes à afficher.
///
/// L'analyse fournit déjà les listes `matched`/`missing` : on ne recalcule pas
/// le score, on nettoie seulement les doublons (casse ignorée, ordre préservé).
fn present_skills(score: &MatchScore) -> Vec<String> {
    clean_skills(&score.matched)
}

/// Compétences de l'offre absentes du profil, prêtes à afficher.
fn missing_skills(score: &MatchScore) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::{missing_skills, present_skills};
    use crate::modules::ia::cv_model::MatchScore;

    fn score(matched: &[&str], missing: &[&str]) -> MatchScore {
        MatchScore {
            total: 0,
            skills: 0,
            experience: 0,
            ats: 0,
            matched: matched.iter().map(|skill| (*skill).to_owned()).collect(),
            missing: missing.iter().map(|skill| (*skill).to_owned()).collect(),
        }
    }

    fn owned(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn les_doublons_sont_supprimes() {
        let analysis = score(&["Rust", "Rust", "Go"], &["SQL", "SQL"]);
        assert_eq!(present_skills(&analysis), owned(&["Rust", "Go"]));
        assert_eq!(missing_skills(&analysis), owned(&["SQL"]));
    }

    #[test]
    fn la_casse_est_ignoree() {
        let analysis = score(&["Rust", "rust"], &["Go", "GO", "go"]);
        assert_eq!(present_skills(&analysis), owned(&["Rust"]));
        assert_eq!(missing_skills(&analysis), owned(&["Go"]));
    }

    #[test]
    fn l_ordre_d_apparition_est_preserve() {
        let analysis = score(&["C", "Rust", "C++"], &["Kafka", "SQL", "Kafka"]);
        assert_eq!(present_skills(&analysis), owned(&["C", "Rust", "C++"]));
        assert_eq!(missing_skills(&analysis), owned(&["Kafka", "SQL"]));
    }

    #[test]
    fn une_liste_vide_reste_vide() {
        let analysis = score(&[], &[]);
        assert!(present_skills(&analysis).is_empty());
        assert!(missing_skills(&analysis).is_empty());
    }
}
