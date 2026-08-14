//! Aperçu natif du CV, construit sur les métriques du template HTML imprimé.

use crate::app::{App, Message};
use crate::core::cv_pdf::{CvExperience, CvPdf, CvProject};
use crate::modules::ia::cv_model::CvGeneration;
use crate::ui::components::button as controls;
use crate::ui::components::{document, surface, typo};
use crate::ui::theme::metrics::{space, stroke};
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use iced::widget::text::LineHeight;
use iced::widget::{column, container, horizontal_rule, image, row, rule, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

/// Largeur A4 de printpdf, en points. Toutes les métriques du preview sont exprimées dans
/// le même repère puis multipliées par le zoom courant.
const PDF_WIDTH_PT: f32 = 595.28;
const CSS_PX_TO_PT: f32 = 0.75;
const CONTENT_X_PT: f32 = 14.17 + 22.4 * CSS_PX_TO_PT;

pub(super) fn preview(app: &App) -> Element<'_, Message> {
    let bar = document::workbench_bar(
        "Aperçu HTML · A4",
        row![
            typo::caption("Template exemple_cv.html"),
            zoom_controls(app),
        ]
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
            document::workspace(document::page_unpadded(app.document_width, page)),
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
    controls::segmented(document::zoom_widths().map(|width| {
        controls::segment(
            format!("{} %", document::zoom_percent(width)),
            (app.document_width - width).abs() < f32::EPSILON,
        )
        .on_press(Message::DocumentWidthChanged(width))
    }))
}

/// Composition partagée avec la bibliothèque. Les marges, tailles et interlignes suivent
/// les valeurs `@media print` du fichier HTML de référence.
pub fn page_content<'a>(
    app: &'a App,
    generation: &'a CvGeneration,
    _title: String,
) -> Element<'a, Message> {
    let cv = crate::modules::ia::cv_document::construire(&app.data.profile, generation);
    let scale = app.document_width / PDF_WIDTH_PT;
    column![cv_header(&cv, scale), cv_body(cv, scale)]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn css(px: f32, scale: f32) -> f32 {
    px * CSS_PX_TO_PT * scale
}

fn pt(value: f32, scale: f32) -> f32 {
    value * scale
}

fn absolute(value: f32) -> LineHeight {
    LineHeight::Absolute(value.into())
}

fn cv_header(document: &CvPdf, scale: f32) -> Element<'static, Message> {
    let mut contacts = row![].spacing(css(14.4, scale));
    for (bytes, value) in contact_items(document) {
        contacts = contacts.push(contact_item(bytes, value, scale));
    }

    container(
        column![
            text(document.name.clone())
                .size(css(32.0, scale))
                .font(font::BOLD)
                .line_height(absolute(css(35.2, scale)))
                .color(cv_text()),
            text(document.subtitle.clone())
                .size(css(13.12, scale))
                .font(font::SEMIBOLD)
                .line_height(absolute(css(18.0, scale)))
                .color(cv_accent()),
            horizontal_rule(1).style(|_| rule::Style {
                color: cv_border(),
                width: 1,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
            }),
            contacts.wrap(),
        ]
        .spacing(css(5.0, scale)),
    )
    .padding([css(10.0, scale), pt(CONTENT_X_PT, scale)])
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(Background::Color(Color::WHITE)),
        border: Border {
            color: cv_border(),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

fn cv_body(document: CvPdf, scale: f32) -> Element<'static, Message> {
    let mut body = column![
        cv_section("Profil", scale),
        cv_paragraph(document.profil.clone(), scale),
    ]
    .spacing(css(4.48, scale));

    if !document.skills.is_empty() {
        body = body
            .push(cv_section("Compétences techniques", scale))
            .push(cv_chips(document.skills.clone(), scale));
    }
    if !document.experiences.is_empty() {
        body = body.push(cv_section("Expérience professionnelle", scale));
        for experience in &document.experiences {
            body = body.push(cv_experience(experience.clone(), scale));
        }
    }
    if !document.projects.is_empty() {
        body = body.push(cv_section("Projets techniques", scale));
        for project in &document.projects {
            body = body.push(cv_project(project.clone(), scale));
        }
    }
    if !document.education.is_empty() || !document.languages.is_empty() {
        body = body.push(cv_education_languages(document, scale));
    }

    container(body)
        .padding([css(8.0, scale), pt(CONTENT_X_PT, scale)])
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::WHITE)),
            ..container::Style::default()
        })
        .into()
}

fn contact_items(document: &CvPdf) -> Vec<(&'static [u8], String)> {
    let mut items = Vec::new();
    if let Some(value) = document
        .phone
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        items.push((
            include_bytes!("../../../../../assets/icons/cv/phone.png").as_slice(),
            value.clone(),
        ));
    }
    if !document.email.trim().is_empty() {
        items.push((
            include_bytes!("../../../../../assets/icons/cv/mail.png").as_slice(),
            document.email.clone(),
        ));
    }
    if let Some(value) = document
        .city
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        items.push((
            include_bytes!("../../../../../assets/icons/cv/pin.png").as_slice(),
            value.clone(),
        ));
    }
    if let Some(value) = document
        .linkedin
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        items.push((
            include_bytes!("../../../../../assets/icons/cv/linkedin.png").as_slice(),
            value.clone(),
        ));
    }
    if let Some(value) = document
        .website
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        items.push((
            include_bytes!("../../../../../assets/icons/cv/globe.png").as_slice(),
            value.clone(),
        ));
    }
    items
}

fn contact_item(bytes: &'static [u8], value: String, scale: f32) -> Element<'static, Message> {
    let icon_size = css(12.0, scale).max(7.0);
    row![
        image(image::Handle::from_bytes(bytes))
            .width(icon_size)
            .height(icon_size),
        text(value)
            .size(css(10.88, scale).max(6.8))
            .font(font::MEDIUM)
            .color(cv_secondary()),
    ]
    .spacing(css(4.2, scale))
    .align_y(Alignment::Center)
    .into()
}

fn cv_section(title: &'static str, scale: f32) -> Element<'static, Message> {
    let width = (title.chars().count() as f32 * css(6.2, scale)).max(css(38.0, scale));
    column![
        text(title.to_uppercase())
            .size(css(9.92, scale))
            .font(font::BOLD)
            .color(cv_accent()),
        container(horizontal_rule(2).style(|_| rule::Style {
            color: cv_accent(),
            width: 2,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
        }))
        .width(width),
    ]
    .spacing(css(1.9, scale))
    .into()
}

fn cv_paragraph(value: String, scale: f32) -> Element<'static, Message> {
    text(value)
        .size(css(12.16, scale))
        .line_height(absolute(css(17.63, scale)))
        .font(font::REGULAR)
        .color(cv_secondary())
        .into()
}

fn cv_chips(skills: Vec<String>, scale: f32) -> Element<'static, Message> {
    let mut line = row![].spacing(css(2.64, scale));
    for skill in skills {
        line = line.push(
            container(
                text(skill)
                    .size(css(10.56, scale))
                    .font(font::MEDIUM)
                    .color(cv_text()),
            )
            .padding([css(1.92, scale), css(6.4, scale)])
            .style(move |_| container::Style {
                background: Some(Background::Color(cv_chip_bg())),
                border: Border {
                    radius: css(4.0, scale).into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }),
        );
    }
    line.wrap().into()
}

fn cv_experience(experience: CvExperience, scale: f32) -> Element<'static, Message> {
    let mut block = column![row![
        column![
            text(experience.title)
                .size(css(13.12, scale))
                .font(font::BOLD)
                .color(cv_text()),
            text(experience.company)
                .size(css(11.2, scale))
                .font(font::REGULAR)
                .color(cv_secondary()),
        ]
        .spacing(css(1.5, scale))
        .width(Length::Fill),
        text(experience.meta)
            .size(css(9.6, scale))
            .font(font::MEDIUM)
            .color(cv_muted()),
    ]
    .spacing(css(8.0, scale))
    .align_y(Alignment::Start),]
    .spacing(css(2.2, scale));
    for bullet in experience.bullets {
        block = block.push(cv_bullet(bullet, scale));
    }
    block.into()
}

fn cv_project(project: CvProject, scale: f32) -> Element<'static, Message> {
    let mut block = column![
        text(project.name)
            .size(css(13.12, scale))
            .font(font::BOLD)
            .color(cv_text()),
        text(project.meta)
            .size(css(11.2, scale))
            .font(font::REGULAR)
            .color(cv_secondary()),
    ]
    .spacing(css(1.8, scale));
    for bullet in project.bullets {
        block = block.push(cv_bullet(bullet, scale));
    }
    block.into()
}

fn cv_bullet(value: String, scale: f32) -> Element<'static, Message> {
    row![
        text("·")
            .size(css(11.52, scale))
            .font(font::REGULAR)
            .color(cv_muted()),
        text(value)
            .size(css(11.52, scale))
            .line_height(absolute(css(15.9, scale)))
            .font(font::REGULAR)
            .color(cv_secondary())
            .width(Length::Fill),
    ]
    .spacing(css(3.0, scale))
    .align_y(Alignment::Start)
    .into()
}

fn cv_education_languages(document: CvPdf, scale: f32) -> Element<'static, Message> {
    let mut left = column![cv_section("Formation", scale)].spacing(css(4.0, scale));
    for education in document.education {
        left = left.push(
            column![
                text(education.degree)
                    .size(css(12.16, scale))
                    .font(font::BOLD)
                    .color(cv_text()),
                text(education.school)
                    .size(css(10.88, scale))
                    .font(font::REGULAR)
                    .color(cv_secondary()),
                text(education.date)
                    .size(css(9.6, scale))
                    .font(font::REGULAR)
                    .color(cv_muted()),
            ]
            .spacing(css(1.5, scale)),
        );
    }

    let mut right = column![cv_section("Disponibilité & langues", scale)].spacing(css(4.0, scale));
    for language in document.languages {
        right = right.push(
            text(format!("{} · {}", language.name, language.level))
                .size(css(12.16, scale))
                .font(font::BOLD)
                .color(cv_text()),
        );
    }

    row![
        left.width(Length::FillPortion(1)),
        right.width(Length::FillPortion(1)),
    ]
    .spacing(css(22.4, scale))
    .align_y(Alignment::Start)
    .into()
}

fn cv_accent() -> Color {
    Color::from_rgb8(0x00, 0x66, 0xcc)
}
fn cv_text() -> Color {
    Color::from_rgb8(0x1a, 0x1a, 0x1a)
}
fn cv_secondary() -> Color {
    Color::from_rgb8(0x3f, 0x3f, 0x46)
}
fn cv_muted() -> Color {
    Color::from_rgb8(0x55, 0x55, 0x5a)
}
fn cv_border() -> Color {
    Color::from_rgb8(0xe2, 0xe2, 0xe5)
}
fn cv_chip_bg() -> Color {
    Color::from_rgb8(0xf5, 0xf5, 0xf7)
}

fn placeholder(app: &App) -> Element<'_, Message> {
    let profile = &app.data.profile.personal;
    let name = format!("{} {}", profile.first_name, profile.last_name)
        .trim()
        .to_owned();
    let scale = app.document_width / PDF_WIDTH_PT;
    let header = container(
        column![
            text(if name.is_empty() {
                "Votre nom".to_owned()
            } else {
                name
            })
            .size(css(32.0, scale))
            .font(font::BOLD)
            .color(cv_text()),
            text(crate::ui::format::or_else(
                profile.headline.as_deref(),
                "Titre professionnel",
            ))
            .size(css(13.12, scale))
            .font(font::SEMIBOLD)
            .color(cv_accent()),
        ]
        .spacing(css(5.0, scale)),
    )
    .padding([css(10.0, scale), pt(CONTENT_X_PT, scale)]);
    let body = container(
        column![
            cv_section("Profil", scale),
            cv_paragraph(
                crate::ui::format::or_else(
                    profile.summary.as_deref(),
                    "Analysez une offre pour générer un CV adapté à sa fiche de poste.",
                ),
                scale,
            ),
        ]
        .spacing(css(4.48, scale)),
    )
    .padding([css(8.0, scale), pt(CONTENT_X_PT, scale)]);
    column![header, body]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
