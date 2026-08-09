//! Écran Profil : sommaire à gauche, section éditable à droite.

use crate::app::state::{Dialog, ProfileSection};
use crate::app::{App, Message};
use crate::modules::profil::components as rows;
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::{badge, inspector, layout, list, state, surface, toolbar, typo};
use crate::ui::format;
use crate::ui::theme::metrics::{size, space};
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{button, column, container, row};
use iced::{Alignment, Element, Length};

/// Rend l'écran du profil.
pub fn view(app: &App) -> Element<'_, Message> {
    let trailing = toolbar::group([controls::primary("Modifier l'identité", Some(Icon::Edit))
        .on_press(Message::OpenDialog(Dialog::Profil))
        .into()]);

    layout::screen(
        toolbar::toolbar(
            "Mon Profil",
            typo::meta("Source structurée des CV et analyses IA"),
            trailing,
        ),
        layout::split_sized(size::SUMMARY, summary(app), section(app)),
    )
}

fn summary(app: &App) -> Element<'_, Message> {
    let mut items = column![].spacing(1).padding(space::MD);
    for section in ProfileSection::ALL {
        let active = app.profile_section == section;
        let count = section_count(app, section);
        let trailing: Element<'_, Message> = if count > 0 {
            badge::count(count)
        } else {
            iced::widget::Space::with_width(0).into()
        };
        items = items.push(
            button(
                row![
                    icon::icon(
                        section_icon(section),
                        icon::MD,
                        if active { Ink::Accent } else { Ink::Muted },
                    ),
                    typo::body(section.label()),
                    layout::spacer(),
                    trailing,
                ]
                .spacing(space::MD)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .height(size::ROW + 2.0)
            .padding([0.0, space::MD])
            .style(styles::nav_item(active))
            .on_press(Message::ProfileSectionChanged(section)),
        );
    }
    container(surface::scroll(items).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(styles::panel_flat)
        .into()
}

const fn section_icon(section: ProfileSection) -> Icon {
    match section {
        ProfileSection::Identite => Icon::Profile,
        ProfileSection::Experiences => Icon::Applications,
        ProfileSection::Competences => Icon::Target,
        ProfileSection::Formations => Icon::Document,
        ProfileSection::Langues => Icon::Link,
        ProfileSection::Import => Icon::Import,
    }
}

fn section_count(app: &App, section: ProfileSection) -> usize {
    let profile = &app.data.profile;
    match section {
        ProfileSection::Identite => usize::from(!profile.personal.email.is_empty()),
        ProfileSection::Experiences => profile.experiences.len(),
        ProfileSection::Competences => profile.skills.len(),
        ProfileSection::Formations => profile.education.len() + profile.certifications.len(),
        ProfileSection::Langues => profile.languages.len(),
        ProfileSection::Import => usize::from(app.extracted_profile.is_some()),
    }
}

fn section(app: &App) -> Element<'_, Message> {
    let profile = &app.data.profile;
    let body: Element<'_, Message> = match app.profile_section {
        ProfileSection::Identite => identity(app),
        ProfileSection::Experiences => collection(
            profile
                .experiences
                .iter()
                .map(rows::experience_row)
                .collect(),
            "Aucune expérience",
            "Importez un CV ou saisissez vos expériences pour nourrir le générateur.",
        ),
        ProfileSection::Competences => skills(app),
        ProfileSection::Formations => {
            let mut items: Vec<Element<'_, Message>> =
                profile.education.iter().map(rows::education_row).collect();
            items.extend(
                profile
                    .certifications
                    .iter()
                    .map(rows::certification_row)
                    .collect::<Vec<_>>(),
            );
            collection(
                items,
                "Aucune formation",
                "Ajoutez vos diplômes et certifications pour compléter vos CV.",
            )
        }
        ProfileSection::Langues => collection(
            profile.languages.iter().map(rows::language_row).collect(),
            "Aucune langue",
            "Précisez vos langues et leur niveau.",
        ),
        ProfileSection::Import => import(app),
    };

    column![
        container(surface::section_header(
            app.profile_section.label(),
            section_actions(app),
        ))
        .height(size::TOOLBAR)
        .padding([0.0, space::XL])
        .align_y(Alignment::Center),
        surface::divider(),
        body,
    ]
    .height(Length::Fill)
    .into()
}

fn section_actions(app: &App) -> Element<'_, Message> {
    match app.profile_section {
        ProfileSection::Identite => controls::ghost("Modifier", Some(Icon::Edit))
            .on_press(Message::OpenDialog(Dialog::Profil))
            .into(),
        ProfileSection::Import => controls::ghost("Choisir un PDF", Some(Icon::Import))
            .on_press(Message::SelectProfilePdf)
            .into(),
        _ => controls::ghost("Modifier le profil", Some(Icon::Edit))
            .on_press(Message::OpenDialog(Dialog::Profil))
            .into(),
    }
}

fn identity(app: &App) -> Element<'_, Message> {
    let personal = &app.data.profile.personal;
    let name = format!("{} {}", personal.first_name, personal.last_name)
        .trim()
        .to_owned();

    surface::scroll(
        container(
            column![
                column![
                    typo::title(if name.is_empty() {
                        "Profil à compléter".to_owned()
                    } else {
                        name
                    }),
                    typo::meta(format::or_else(
                        personal.headline.as_deref(),
                        "Titre professionnel non renseigné"
                    )),
                ]
                .spacing(1),
                inspector::group(
                    "Coordonnées",
                    [
                        inspector::property("E-mail", format::or_dash(Some(&personal.email))),
                        inspector::property(
                            "Téléphone",
                            format::or_dash(personal.phone.as_deref())
                        ),
                        inspector::property("Ville", format::or_dash(personal.city.as_deref())),
                    ],
                ),
                inspector::group(
                    "Présence en ligne",
                    [
                        inspector::property(
                            "LinkedIn",
                            format::or_dash(personal.linkedin.as_deref())
                        ),
                        inspector::property("GitHub", format::or_dash(personal.github.as_deref())),
                        inspector::property(
                            "Site web",
                            format::or_dash(personal.website.as_deref())
                        ),
                    ],
                ),
                inspector::note("Résumé", personal.summary.clone()),
            ]
            .spacing(space::XXL),
        )
        .padding(space::XL),
    )
    .height(Length::Fill)
    .into()
}

fn skills(app: &App) -> Element<'_, Message> {
    if app.data.profile.skills.is_empty() {
        return state::empty(
            "Aucune compétence",
            "Les compétences alimentent le score de correspondance avec les offres.",
        );
    }
    let names: Vec<String> = app
        .data
        .profile
        .skills
        .iter()
        .map(|skill| skill.name.clone())
        .collect();
    let mut grid = row![].spacing(space::SM);
    for name in names {
        grid = grid.push(badge::badge(name, Tone::Accent));
    }
    surface::scroll(container(grid.wrap()).padding(space::XL))
        .height(Length::Fill)
        .into()
}

fn collection<'a>(
    items: Vec<Element<'a, Message>>,
    empty_title: &'a str,
    empty_hint: &'a str,
) -> Element<'a, Message> {
    if items.is_empty() {
        return state::empty(empty_title, empty_hint);
    }
    let mut body = column![];
    for item in items {
        body = body.push(item);
    }
    surface::scroll(body).height(Length::Fill).into()
}

fn import(app: &App) -> Element<'_, Message> {
    let file = app.profile_import_path.as_ref().map_or_else(
        || "Aucun CV sélectionné".to_owned(),
        |path| {
            path.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("cv.pdf")
                .to_owned()
        },
    );

    let mut body = column![
        state::hint("Le PDF est lu localement ; rien n'est enregistré avant votre validation."),
        list::row_static(
            icon::muted(Icon::Document),
            typo::body(file),
            controls::ghost("Choisir un PDF", Some(Icon::Import))
                .on_press(Message::SelectProfilePdf),
        ),
    ]
    .spacing(space::LG);

    body = body.push(if app.ai_is_running {
        state::running(
            "Extraction du profil",
            app.ai_elapsed_seconds,
            Message::CancelAi,
        )
    } else {
        row![
            layout::spacer(),
            controls::secondary("Analyser le CV", Some(Icon::Sparkles))
                .on_press(Message::ExtractProfile),
        ]
        .align_y(Alignment::Center)
        .into()
    });

    if let Some(extracted) = &app.extracted_profile {
        body = body
            .push(surface::divider())
            .push(typo::label("Profil détecté"))
            .push(inspector::property(
                "Identité",
                format!(
                    "{} {}",
                    extracted.personal.first_name, extracted.personal.last_name
                ),
            ))
            .push(inspector::property(
                "Expériences",
                extracted.experiences.len().to_string(),
            ))
            .push(inspector::property(
                "Compétences",
                extracted.skills.len().to_string(),
            ))
            .push(inspector::property(
                "Formations",
                extracted.education.len().to_string(),
            ))
            .push(state::hint(
                "Valider remplacera le profil actuel par le profil détecté.",
            ))
            .push(
                row![
                    layout::spacer(),
                    controls::primary("Valider et remplacer", Some(Icon::Check))
                        .on_press(Message::ApplyExtractedProfile),
                ]
                .align_y(Alignment::Center),
            );
    }

    surface::scroll(container(body).padding(space::XL))
        .height(Length::Fill)
        .into()
}
