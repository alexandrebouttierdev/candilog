//! Sections éditables du profil professionnel.

use crate::app::state::{Dialog, ProfileCollection, ProfileSection};
use crate::app::{App, Message};
use crate::modules::profil::components as rows;
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::{badge, inspector, layout, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::{radius, size, space};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, text};
use iced::{Alignment, Background, Border, Element, Length, Theme};

const HEADER_TILE: f32 = 28.0;

/// Composition éditoriale du profil : identité et compétences en premier,
/// parcours au centre, éléments complémentaires ensuite.
pub(super) fn sections_grid(app: &App) -> Element<'_, Message> {
    column![
        identity_section(app),
        row![
            column![
                experiences_section(app),
                skills_section(app),
                languages_section(app),
            ]
            .spacing(space::LG)
            .width(Length::FillPortion(1)),
            column![
                formations_section(app),
                projects_section(app),
                certifications_section(app),
            ]
            .spacing(space::LG)
            .width(Length::FillPortion(1)),
        ]
        .spacing(space::LG),
    ]
    .spacing(space::LG)
    .width(Length::Fill)
    .into()
}

/// Carte d'une section : en-tête (icône, titre, compteur, crayon) et contenu.
fn section_card<'a>(
    glyph: Icon,
    title: &'a str,
    count: Option<usize>,
    edit: Option<ProfileSection>,
    content: Element<'a, Message>,
) -> Element<'a, Message> {
    let mut header = row![
        header_tile(glyph),
        text(title).size(font::LABEL).font(font::SEMIBOLD),
        layout::spacer(),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center);
    if let Some(count) = count {
        header = header.push(typo::text_mono(
            count.to_string(),
            font::MICRO,
            font::MONO_SEMIBOLD,
        ));
    }
    if let Some(section) = edit {
        header = header.push(controls::icon_action(
            Icon::Edit,
            "Modifier",
            Message::OpenDialog(Dialog::Profil(section)),
        ));
    }

    container(
        column![
            container(header)
                .height(size::SECTION_HEADER)
                .align_y(Alignment::Center),
            surface::divider(),
            content,
        ]
        .width(Length::Fill),
    )
    .padding([space::MD, space::LG])
    .width(Length::Fill)
    .style(styles::glass_card)
    .into()
}

/// Pastille d'icône d'un en-tête de section (`bg-secondary/70 rounded-xl`).
pub(super) fn header_tile<'a, Message: 'a>(glyph: Icon) -> Element<'a, Message> {
    container(icon::icon(glyph, icon::SM, Ink::Muted))
        .width(HEADER_TILE)
        .height(HEADER_TILE)
        .center(Length::Fixed(HEADER_TILE))
        .style(|theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(alpha(palette.sunken, 0.70))),
                border: Border {
                    radius: radius::CONTROL.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Contenu d'une section : les lignes, ou un emplacement vide minimal.
fn rows_or_empty<'a, Message: 'a>(
    items: Vec<Element<'a, Message>>,
    hint: &'a str,
) -> Element<'a, Message> {
    if items.is_empty() {
        return state::empty_slot(hint);
    }
    let mut body = column![].width(Length::Fill);
    for item in items {
        body = body.push(item);
    }
    body.into()
}

/// Carte Identité : coordonnées et présence en ligne.
fn identity_section(app: &App) -> Element<'_, Message> {
    let personal = &app.data.profile.personal;
    let filled = [
        !personal.email.is_empty(),
        personal
            .phone
            .as_deref()
            .is_some_and(|value| !value.is_empty()),
        personal
            .city
            .as_deref()
            .is_some_and(|value| !value.is_empty()),
        personal
            .linkedin
            .as_deref()
            .is_some_and(|value| !value.is_empty()),
        personal
            .github
            .as_deref()
            .is_some_and(|value| !value.is_empty()),
        personal
            .website
            .as_deref()
            .is_some_and(|value| !value.is_empty()),
    ];
    let count = filled.into_iter().filter(|filled| *filled).count();

    section_card(
        Icon::Profile,
        "Identité",
        Some(count),
        Some(ProfileSection::Identite),
        container(
            row![
                container(inspector::group(
                    "Coordonnées",
                    [
                        inspector::property("E-mail", format::or_dash(Some(&personal.email))),
                        inspector::property(
                            "Téléphone",
                            format::or_dash(personal.phone.as_deref()),
                        ),
                        inspector::property("Ville", format::or_dash(personal.city.as_deref())),
                    ],
                ))
                .width(Length::FillPortion(1)),
                container(inspector::group(
                    "Présence en ligne",
                    [
                        inspector::property(
                            "LinkedIn",
                            format::or_dash(personal.linkedin.as_deref()),
                        ),
                        inspector::property("GitHub", format::or_dash(personal.github.as_deref())),
                        inspector::property(
                            "Site web",
                            format::or_dash(personal.website.as_deref()),
                        ),
                    ],
                ))
                .width(Length::FillPortion(1)),
                container(inspector::note("Résumé", personal.summary.clone()))
                    .width(Length::FillPortion(1)),
            ]
            .spacing(space::XXL)
            .align_y(Alignment::Start),
        )
        .padding([space::MD, 0.0])
        .into(),
    )
}

/// Carte Expériences : la timeline est approchée par des lignes dont la
/// période porte un jeton monospace.
fn experiences_section(app: &App) -> Element<'_, Message> {
    let experiences = &app.data.profile.experiences;
    section_card(
        Icon::Applications,
        "Expériences",
        Some(experiences.len()),
        Some(ProfileSection::Collection(ProfileCollection::Experience)),
        rows_or_empty(
            experiences.iter().map(rows::experience_row).collect(),
            "Aucune expérience — importez un CV ou ouvrez le dialogue Profil.",
        ),
    )
}

/// Carte Compétences : jetons à plat.
fn skills_section(app: &App) -> Element<'_, Message> {
    let skills = &app.data.profile.skills;
    let content: Element<'_, Message> = if skills.is_empty() {
        state::empty_slot("Aucune compétence — ajoutez-en dans le dialogue Profil.")
    } else {
        let mut line = row![].spacing(space::SM).width(Length::Fill);
        for skill in skills {
            line = line.push(badge::badge(skill.name.clone(), Tone::Accent));
        }
        container(line.wrap()).padding(space::MD).into()
    };
    section_card(
        Icon::Target,
        "Compétences",
        Some(skills.len()),
        Some(ProfileSection::Competences),
        content,
    )
}

/// Carte Formations : diplômes (les certifications ont leur propre carte).
fn formations_section(app: &App) -> Element<'_, Message> {
    let profile = &app.data.profile;
    let items: Vec<Element<'_, Message>> =
        profile.education.iter().map(rows::education_row).collect();
    section_card(
        Icon::Document,
        "Formations",
        Some(profile.education.len()),
        Some(ProfileSection::Collection(ProfileCollection::Formation)),
        rows_or_empty(items, "Aucune formation — ajoutez vos diplômes."),
    )
}

/// Carte Langues : langues parlées et niveau.
fn languages_section(app: &App) -> Element<'_, Message> {
    let languages = &app.data.profile.languages;
    section_card(
        Icon::Link,
        "Langues",
        Some(languages.len()),
        Some(ProfileSection::Collection(ProfileCollection::Langue)),
        rows_or_empty(
            languages.iter().map(rows::language_row).collect(),
            "Aucune langue — précisez vos langues et leur niveau.",
        ),
    )
}

/// Carte Projets : projets personnels et professionnels.
fn projects_section(app: &App) -> Element<'_, Message> {
    let projects = &app.data.profile.projects;
    section_card(
        Icon::Network,
        "Projets",
        Some(projects.len()),
        Some(ProfileSection::Collection(ProfileCollection::Projet)),
        rows_or_empty(
            projects.iter().map(rows::project_row).collect(),
            "Aucun projet — ajoutez vos réalisations.",
        ),
    )
}

/// Carte Certifications : certifications obtenues.
fn certifications_section(app: &App) -> Element<'_, Message> {
    let certifications = &app.data.profile.certifications;
    section_card(
        Icon::Check,
        "Certifications",
        Some(certifications.len()),
        Some(ProfileSection::Collection(ProfileCollection::Certification)),
        rows_or_empty(
            certifications.iter().map(rows::certification_row).collect(),
            "Aucune certification — ajoutez vos attestations.",
        ),
    )
}
