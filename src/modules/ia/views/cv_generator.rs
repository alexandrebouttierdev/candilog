//! CV Generator : atelier d'analyse à gauche, document A4 posé à droite.

use crate::app::state::RecommendationStatus;
use crate::app::{App, Message};
use crate::modules::ia::cv_model::CvGeneration;
use crate::ui::components::button as controls;
use crate::ui::components::icon::Icon;
use crate::ui::components::{badge, document, field, layout, meter, state, surface, toolbar, typo};
use crate::ui::theme::metrics::space;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Rend l'écran du générateur de CV.
pub fn view(app: &App) -> Element<'_, Message> {
    let status = if app.ai_is_running {
        badge::badge("Analyse en cours", Tone::Warning)
    } else if app.cv_generation.is_some() {
        badge::badge("CV généré", Tone::Success)
    } else {
        badge::badge("Prêt", Tone::Neutral)
    };

    let trailing = toolbar::group([
        field::input("Nom de la version", &app.cv_version_name)
            .on_input(Message::CvVersionNameChanged)
            .width(Length::Fixed(180.0))
            .into(),
        controls::ghost("Exporter PDF", Some(Icon::Download))
            .on_press(Message::ExportGeneratedCvPdf)
            .into(),
        controls::primary("Enregistrer la version", Some(Icon::Save))
            .on_press(Message::SaveGeneratedCv)
            .into(),
    ]);

    layout::screen(
        toolbar::toolbar("CV Generator", status, trailing),
        layout::split_portions(5, workbench(app), 6, preview(app)),
    )
}

fn workbench(app: &App) -> Element<'_, Message> {
    let mut panel = column![
        surface::section_header(
            "Offre ciblée",
            typo::caption("Collez le texte complet de l'annonce"),
        ),
        surface::divider(),
        field::editor(&app.offer_editor, "Collez ici le texte complet de l'offre…")
            .on_action(Message::OfferEditorAction)
            .height(Length::Fixed(150.0)),
    ]
    .spacing(space::LG);

    panel = panel.push(if app.ai_is_running {
        state::running(
            "Analyse de l'offre",
            app.ai_elapsed_seconds,
            Message::CancelAi,
        )
    } else {
        row![
            layout::spacer(),
            controls::secondary("Analyser l'offre", Some(Icon::Target))
                .on_press(Message::AnalyzeOffer),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center)
        .into()
    });

    if let Some(analysis) = &app.offer_analysis {
        panel = panel
            .push(surface::divider())
            .push(surface::section_header(
                "Correspondance",
                badge::badge(
                    format!("{} / 100", analysis.score.total),
                    score_tone(analysis.score.total),
                ),
            ))
            .push(
                row![
                    meter::bar(
                        "Compétences".into(),
                        usize::from(analysis.score.skills),
                        100,
                        Tone::Accent,
                    ),
                    meter::bar(
                        "Expérience".into(),
                        usize::from(analysis.score.experience),
                        100,
                        Tone::Info,
                    ),
                    meter::bar(
                        "Mots-clés ATS".into(),
                        usize::from(analysis.score.ats),
                        100,
                        Tone::Success,
                    ),
                ]
                .spacing(space::XL),
            )
            .push(crate::modules::ia::components::skill_list(
                "Présentes dans votre profil",
                &analysis.score.matched,
                Tone::Success,
            ))
            .push(crate::modules::ia::components::skill_list(
                "Manquantes",
                &analysis.score.missing,
                Tone::Warning,
            ))
            .push(
                row![
                    layout::spacer(),
                    controls::secondary("Générer le CV", Some(Icon::Sparkles))
                        .on_press(Message::GenerateCv),
                ]
                .align_y(Alignment::Center),
            );
    }

    if let Some(generation) = &app.cv_generation {
        panel = panel
            .push(surface::divider())
            .push(surface::section_header(
                "Suggestions ATS",
                badge::count(generation.analysis.recommandations.len()),
            ))
            .push(meter::ats(generation.analysis.score));
        if generation.analysis.recommandations.is_empty() {
            panel = panel.push(state::empty_slot("Aucune suggestion à appliquer."));
        }
        for (index, recommendation) in generation.analysis.recommandations.iter().enumerate() {
            let status = app
                .recommendation_states
                .get(index)
                .copied()
                .unwrap_or(RecommendationStatus::Pending);
            panel = panel.push(crate::modules::ia::components::recommendation(
                recommendation.section.clone(),
                recommendation.impact,
                recommendation.texte_original.clone(),
                recommendation.texte_propose.clone(),
                status,
                Message::AcceptRecommendation(index),
                Message::RejectRecommendation(index),
            ));
        }
    }

    surface::scroll(container(panel).padding(space::XL))
        .height(Length::Fill)
        .into()
}

const fn score_tone(score: u8) -> Tone {
    crate::ui::components::meter::ats_level(score).tone()
}

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

    column![
        container(bar).width(Length::Fill),
        surface::divider(),
        document::workspace(document::page(app.document_width, page)),
    ]
    .height(Length::Fill)
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
