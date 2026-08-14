//! Formulaires d'édition des sections du profil (identité, compétences et
//! collections).

use crate::app::state::{ProfileCollection, ProfileSection};
use crate::app::{App, Message};
use crate::ui::components::button as controls;
use crate::ui::components::icon::Icon;
use crate::ui::components::{badge, field, layout, state, typo};
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Titre du dialogue selon la section éditée.
pub fn dialog_title(section: ProfileSection) -> &'static str {
    match section {
        ProfileSection::Identite => "Modifier l'identité",
        ProfileSection::Competences => "Modifier les compétences",
        ProfileSection::Collection(ProfileCollection::Experience) => "Modifier les expériences",
        ProfileSection::Collection(ProfileCollection::Formation) => "Modifier les formations",
        ProfileSection::Collection(ProfileCollection::Langue) => "Modifier les langues",
        ProfileSection::Collection(ProfileCollection::Projet) => "Modifier les projets",
        ProfileSection::Collection(ProfileCollection::Certification) => {
            "Modifier les certifications"
        }
    }
}

/// Formulaire de la section de profil demandée.
pub fn form(app: &App, section: ProfileSection) -> Element<'_, Message> {
    match section {
        ProfileSection::Identite => identity(app),
        ProfileSection::Competences => skills(app),
        ProfileSection::Collection(collection) => collection_editor(app, collection),
    }
}

fn identity(app: &App) -> Element<'_, Message> {
    let personal = &app.profile_personal_form;
    column![
        field::form_row([
            field::text_field(
                "Prénom",
                &personal.first_name,
                Message::ProfileFirstNameChanged,
            ),
            field::text_field("Nom", &personal.last_name, Message::ProfileLastNameChanged),
        ]),
        field::form_row([
            field::text_field("E-mail", &personal.email, Message::ProfileEmailChanged),
            field::text_field(
                "Téléphone",
                personal.phone.as_deref().unwrap_or_default(),
                Message::ProfilePhoneChanged,
            ),
        ]),
        field::form_row([
            field::text_field(
                "Ville",
                personal.city.as_deref().unwrap_or_default(),
                Message::ProfileCityChanged,
            ),
            field::text_field(
                "Titre professionnel",
                personal.headline.as_deref().unwrap_or_default(),
                Message::ProfileHeadlineChanged,
            ),
        ]),
        field::form_row([
            field::text_field(
                "LinkedIn",
                personal.linkedin.as_deref().unwrap_or_default(),
                Message::ProfileLinkedinChanged,
            ),
            field::text_field(
                "GitHub",
                personal.github.as_deref().unwrap_or_default(),
                Message::ProfileGithubChanged,
            ),
        ]),
        field::text_field(
            "Site web",
            personal.website.as_deref().unwrap_or_default(),
            Message::ProfileWebsiteChanged,
        ),
        field::labeled(
            "Résumé",
            field::editor(
                &app.profile_summary_editor,
                "Présentez votre parcours, vos points forts et votre objectif…"
            )
            .on_action(Message::ProfileSummaryChanged)
            .height(Length::Fixed(132.0)),
        ),
        typo::meta_toned(
            "Ces informations alimentent le générateur de CV et le score ATS.",
            Tone::Neutral,
        ),
    ]
    .spacing(space::LG)
    .into()
}

fn skills(app: &App) -> Element<'_, Message> {
    column![
        row![
            field::input("Nouvelle compétence", &app.profile_skills_form)
                .on_input(Message::ProfileSkillsChanged)
                .on_submit(Message::ProfileSkillAdded)
                .width(Length::Fill),
            controls::secondary("Ajouter", Some(Icon::Plus)).on_press(Message::ProfileSkillAdded),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
        skills_editor(app),
        typo::meta_toned(
            "Ces informations alimentent le générateur de CV et le score ATS.",
            Tone::Neutral,
        ),
    ]
    .spacing(space::LG)
    .into()
}

fn skills_editor(app: &App) -> Element<'_, Message> {
    if app.profile_draft.skills.is_empty() {
        return state::empty_slot("Ajoutez vos compétences une par une.");
    }
    let mut line = row![].spacing(space::SM).width(Length::Fill);
    for (index, skill) in app.profile_draft.skills.iter().enumerate() {
        line = line.push(
            row![
                badge::badge(skill.name.clone(), Tone::Accent),
                controls::icon_danger(Icon::Close, "Retirer", Message::ProfileSkillRemoved(index),),
            ]
            .spacing(space::XXS)
            .align_y(Alignment::Center),
        );
    }
    line.wrap().into()
}

fn heading<'a>(title: &'a str, add: Option<Message>) -> Element<'a, Message> {
    let mut line = row![typo::title(title), layout::spacer()].align_y(Alignment::Center);
    if let Some(message) = add {
        line = line.push(controls::secondary("Ajouter", Some(Icon::Plus)).on_press(message));
    }
    line.into()
}

fn collection_editor(app: &App, kind: ProfileCollection) -> Element<'_, Message> {
    let items: Vec<Element<'_, Message>> = match kind {
        ProfileCollection::Experience => app
            .profile_draft
            .experiences
            .iter()
            .enumerate()
            .map(|(index, item)| {
                item_form(
                    kind,
                    index,
                    vec![
                        value("Poste", &item.title, kind, index, 0),
                        value("Entreprise", &item.company, kind, index, 1),
                        value(
                            "Lieu",
                            item.location.as_deref().unwrap_or_default(),
                            kind,
                            index,
                            2,
                        ),
                        value("Début", &item.start_date, kind, index, 3),
                        value(
                            "Fin (vide = en cours)",
                            item.end_date.as_deref().unwrap_or_default(),
                            kind,
                            index,
                            4,
                        ),
                        value(
                            "Description",
                            item.description.as_deref().unwrap_or_default(),
                            kind,
                            index,
                            5,
                        ),
                    ],
                )
            })
            .collect(),
        ProfileCollection::Formation => app
            .profile_draft
            .education
            .iter()
            .enumerate()
            .map(|(index, item)| {
                item_form(
                    kind,
                    index,
                    vec![
                        value("Diplôme", &item.degree, kind, index, 0),
                        value("Établissement", &item.school, kind, index, 1),
                        value(
                            "Lieu",
                            item.location.as_deref().unwrap_or_default(),
                            kind,
                            index,
                            2,
                        ),
                        value(
                            "Début",
                            item.start_date.as_deref().unwrap_or_default(),
                            kind,
                            index,
                            3,
                        ),
                        value(
                            "Fin",
                            item.end_date.as_deref().unwrap_or_default(),
                            kind,
                            index,
                            4,
                        ),
                        value(
                            "Description",
                            item.description.as_deref().unwrap_or_default(),
                            kind,
                            index,
                            5,
                        ),
                    ],
                )
            })
            .collect(),
        ProfileCollection::Langue => app
            .profile_draft
            .languages
            .iter()
            .enumerate()
            .map(|(index, item)| {
                item_form(
                    kind,
                    index,
                    vec![
                        value("Langue", &item.name, kind, index, 0),
                        value("Niveau", &item.level, kind, index, 1),
                    ],
                )
            })
            .collect(),
        ProfileCollection::Projet => app
            .profile_draft
            .projects
            .iter()
            .enumerate()
            .map(|(index, item)| {
                item_form(
                    kind,
                    index,
                    vec![
                        value("Nom", &item.name, kind, index, 0),
                        value(
                            "Lien",
                            item.url.as_deref().unwrap_or_default(),
                            kind,
                            index,
                            1,
                        ),
                        value(
                            "Technologies",
                            item.technologies.as_deref().unwrap_or_default(),
                            kind,
                            index,
                            2,
                        ),
                        value(
                            "Description",
                            item.description.as_deref().unwrap_or_default(),
                            kind,
                            index,
                            3,
                        ),
                    ],
                )
            })
            .collect(),
        ProfileCollection::Certification => app
            .profile_draft
            .certifications
            .iter()
            .enumerate()
            .map(|(index, item)| {
                item_form(
                    kind,
                    index,
                    vec![
                        value("Nom", &item.name, kind, index, 0),
                        value(
                            "Organisme",
                            item.issuer.as_deref().unwrap_or_default(),
                            kind,
                            index,
                            1,
                        ),
                        value(
                            "Date",
                            item.date.as_deref().unwrap_or_default(),
                            kind,
                            index,
                            2,
                        ),
                        value(
                            "Lien",
                            item.url.as_deref().unwrap_or_default(),
                            kind,
                            index,
                            3,
                        ),
                    ],
                )
            })
            .collect(),
    };
    let title = match kind {
        ProfileCollection::Experience => "Expériences",
        ProfileCollection::Formation => "Formations",
        ProfileCollection::Langue => "Langues",
        ProfileCollection::Projet => "Projets",
        ProfileCollection::Certification => "Certifications",
    };
    let content = if items.is_empty() {
        state::empty_slot("Aucune entrée pour le moment.")
    } else {
        let mut list = column![].spacing(space::SM);
        for item in items {
            list = list.push(item);
        }
        list.into()
    };
    column![
        heading(title, Some(Message::ProfileItemAdded(kind))),
        content,
    ]
    .spacing(space::SM)
    .into()
}

fn item_form<'a>(
    kind: ProfileCollection,
    index: usize,
    fields: Vec<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut body = column![].spacing(space::SM);
    let mut fields = fields.into_iter();
    while let Some(first) = fields.next() {
        let mut pair = vec![first];
        if let Some(second) = fields.next() {
            pair.push(second);
        }
        body = body.push(field::form_row(pair));
    }
    container(
        column![
            row![
                typo::caption(format!("Entrée {}", index + 1)),
                layout::spacer(),
                controls::icon_danger(
                    Icon::Trash,
                    "Supprimer",
                    Message::ProfileItemRemoved(kind, index),
                ),
            ]
            .align_y(Alignment::Center),
            body,
        ]
        .spacing(space::SM),
    )
    .padding(space::MD)
    .width(Length::Fill)
    .style(styles::sunken)
    .into()
}

fn value<'a>(
    label: &'a str,
    value: &'a str,
    kind: ProfileCollection,
    index: usize,
    field_index: usize,
) -> Element<'a, Message> {
    field::text_field(label, value, move |value| {
        Message::ProfileItemChanged(kind, index, field_index, value)
    })
}
