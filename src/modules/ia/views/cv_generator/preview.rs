//! Aperçu natif et composition de la page A4 du CV.

use crate::app::{App, Message};
use crate::core::cv_pdf::{CvExperience, CvPdf, CvProject};
use crate::modules::ia::cv_model::CvGeneration;
use crate::ui::components::button as controls;
use crate::ui::components::{document, surface, typo};
use crate::ui::theme::metrics::{space, stroke};
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use iced::widget::{column, container, horizontal_rule, row, rule, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

/// Aperçu du document A4, posé sur son plan de travail.
pub(super) fn preview(app: &App) -> Element<'_, Message> {
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
            background: Some(Background::Color(palette.panel)),
            border: Border {
                color: palette.border,
                width: stroke::HAIRLINE,
                ..Border::default()
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

/// Contenu de la page A4, fidèle au design du PDF exporté.
pub fn page_content<'a>(
    app: &'a App,
    generation: &'a CvGeneration,
    _title: String,
) -> Element<'a, Message> {
    let document = crate::modules::ia::cv_document::construire(&app.data.profile, generation);
    let mut page = column![
        cv_nom(document.name.clone()),
        cv_sous_titre(document.subtitle.clone()),
        cv_contact(&document),
        iced::widget::Space::with_height(space::SM),
        cv_section("Profil"),
        cv_paragraphe(document.profil.clone()),
    ]
    .spacing(space::SM)
    .width(Length::Fill);

    if !document.skills.is_empty() {
        page = page
            .push(iced::widget::Space::with_height(space::SM))
            .push(cv_section("Compétences techniques"))
            .push(cv_chips(document.skills.clone()));
    }

    if !document.experiences.is_empty() {
        page = page
            .push(iced::widget::Space::with_height(space::SM))
            .push(cv_section("Expérience professionnelle"));
        for experience in &document.experiences {
            page = page.push(cv_experience(experience.clone()));
        }
    }

    if !document.projects.is_empty() {
        page = page
            .push(iced::widget::Space::with_height(space::SM))
            .push(cv_section("Projets techniques"));
        for projet in &document.projects {
            page = page.push(cv_projet(projet.clone()));
        }
    }

    if !document.education.is_empty() || !document.languages.is_empty() {
        page = page
            .push(iced::widget::Space::with_height(space::SM))
            .push(cv_formation_langues(document));
    }

    page.into()
}

fn cv_accent() -> Color {
    Color::from_rgb8(0x00, 0x66, 0xcc)
}

fn cv_texte() -> Color {
    Color::from_rgb8(0x1a, 0x1a, 0x1a)
}

fn cv_secondaire() -> Color {
    Color::from_rgb8(0x3f, 0x3f, 0x46)
}

fn cv_muted() -> Color {
    Color::from_rgb8(0x55, 0x55, 0x5a)
}

fn cv_chip_bg() -> Color {
    Color::from_rgb8(0xf5, 0xf5, 0xf7)
}

fn cv_nom(nom: String) -> Element<'static, Message> {
    text(nom)
        .size(20.0)
        .font(font::SEMIBOLD)
        .color(cv_texte())
        .into()
}

fn cv_sous_titre(sous_titre: String) -> Element<'static, Message> {
    text(sous_titre)
        .size(10.0)
        .font(font::MEDIUM)
        .color(cv_accent())
        .into()
}

fn cv_contact(document: &CvPdf) -> Element<'static, Message> {
    let contact = [
        document.phone.as_deref(),
        Some(document.email.as_str()),
        document.city.as_deref(),
        document.linkedin.as_deref(),
        document.website.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter(|valeur| !valeur.trim().is_empty())
    .collect::<Vec<_>>()
    .join(" · ");
    text(contact)
        .size(8.0)
        .font(font::REGULAR)
        .color(cv_secondaire())
        .into()
}

fn cv_section(titre: &'static str) -> Element<'static, Message> {
    column![
        text(titre.to_uppercase())
            .size(8.0)
            .font(font::SEMIBOLD)
            .color(cv_accent()),
        horizontal_rule(2).style(|_| rule::Style {
            color: cv_accent(),
            width: 2,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
        }),
    ]
    .spacing(2.0)
    .into()
}

fn cv_paragraphe(valeur: String) -> Element<'static, Message> {
    text(valeur)
        .size(8.5)
        .font(font::REGULAR)
        .color(cv_secondaire())
        .into()
}

fn cv_chips(competences: Vec<String>) -> Element<'static, Message> {
    let mut ligne = row![].spacing(4.0);
    for competence in competences {
        ligne = ligne.push(
            container(
                text(competence)
                    .size(7.0)
                    .font(font::MEDIUM)
                    .color(cv_texte()),
            )
            .padding([2.0, 5.0])
            .style(|_| container::Style {
                background: Some(Background::Color(cv_chip_bg())),
                border: Border {
                    radius: 3.0.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }),
        );
    }
    ligne.wrap().into()
}

fn cv_experience(experience: CvExperience) -> Element<'static, Message> {
    let mut meta = experience.company.clone();
    if !experience.meta.is_empty() {
        meta = format!("{} · {}", meta, experience.meta);
    }
    let mut bloc = column![
        text(experience.title)
            .size(9.0)
            .font(font::SEMIBOLD)
            .color(cv_texte()),
        text(meta)
            .size(8.0)
            .font(font::REGULAR)
            .color(cv_secondaire()),
    ]
    .spacing(2.0);
    for puce in experience.bullets {
        bloc = bloc.push(cv_puce(puce));
    }
    bloc.into()
}

fn cv_projet(projet: CvProject) -> Element<'static, Message> {
    let mut bloc = column![text(projet.name)
        .size(9.0)
        .font(font::SEMIBOLD)
        .color(cv_texte())]
    .spacing(2.0);
    if !projet.meta.is_empty() {
        bloc = bloc.push(
            text(projet.meta)
                .size(8.0)
                .font(font::REGULAR)
                .color(cv_secondaire()),
        );
    }
    for puce in projet.bullets {
        bloc = bloc.push(cv_puce(puce));
    }
    bloc.into()
}

fn cv_puce(valeur: String) -> Element<'static, Message> {
    row![
        text("·").size(7.5).font(font::REGULAR).color(cv_muted()),
        text(valeur)
            .size(7.5)
            .font(font::REGULAR)
            .color(cv_secondaire())
            .width(Length::Fill),
    ]
    .spacing(4.0)
    .align_y(Alignment::Start)
    .into()
}

fn cv_formation_langues(document: CvPdf) -> Element<'static, Message> {
    let mut gauche = column![cv_section("Formation")].spacing(4.0);
    for education in document.education {
        let mut bloc = column![
            text(education.degree)
                .size(8.5)
                .font(font::SEMIBOLD)
                .color(cv_texte()),
            text(education.school)
                .size(7.5)
                .font(font::REGULAR)
                .color(cv_secondaire()),
        ]
        .spacing(1.5);
        if !education.date.is_empty() {
            bloc = bloc.push(
                text(education.date)
                    .size(7.0)
                    .font(font::REGULAR)
                    .color(cv_muted()),
            );
        }
        gauche = gauche.push(bloc);
    }

    let mut droite = column![cv_section("Disponibilité & langues")].spacing(4.0);
    for langue in document.languages {
        droite = droite.push(
            text(format!("{} · {}", langue.name, langue.level))
                .size(8.5)
                .font(font::SEMIBOLD)
                .color(cv_texte()),
        );
    }

    row![
        gauche.width(Length::FillPortion(1)),
        droite.width(Length::FillPortion(1)),
    ]
    .spacing(16.0)
    .align_y(Alignment::Start)
    .into()
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
