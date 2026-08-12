//! Écran Profil : identité, complétion et sections en cartes sur une seule
//! page défilante, dans l'esprit candilog-desktop.

use crate::app::state::{Dialog, ProfileCollection, ProfileSection};
use crate::app::{App, Message};
use crate::modules::profil::components as rows;
use crate::navigation::Route;
use crate::shared::profile::{PersonalInfo, Profile};
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::{badge, inspector, layout, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{column, container, progress_bar, row, text, Space};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Largeur de la colonne de complétion, à droite de la carte d'identité
/// (`w-36`).
const COMPLETION_WIDTH: f32 = 144.0;
/// Côté de la pastille d'icône d'un en-tête de section (`w-7`).
const HEADER_TILE: f32 = 28.0;

/// Score de complétion du profil (0-100), 7 sections pondérées également.
#[must_use]
pub fn completion_score(profile: &Profile) -> u8 {
    let complete = [
        identity_complete(&profile.personal),
        profile.experiences.iter().any(|item| item.is_complete()),
        profile.skills.iter().any(|item| item.is_complete()),
        profile.education.iter().any(|item| item.is_complete()),
        profile.languages.iter().any(|item| item.is_complete()),
        profile.projects.iter().any(|item| item.is_complete()),
        profile.certifications.iter().any(|item| item.is_complete()),
    ]
    .into_iter()
    .filter(|complete| *complete)
    .count() as u16;
    // Arrondi au plus proche : `+ 3` (moitié de 7) avant la division.
    ((complete * 100 + 3) / 7) as u8
}

/// La section identité est complète quand nom, prénom et e-mail sont remplis.
fn identity_complete(personal: &PersonalInfo) -> bool {
    !personal.first_name.trim().is_empty()
        && !personal.last_name.trim().is_empty()
        && !personal.email.trim().is_empty()
}

/// Rend l'écran du profil.
pub fn view(app: &App) -> Element<'_, Message> {
    layout::screen(
        header::route_header(
            Icon::Profile,
            "Profil professionnel",
            Route::Profil,
            Message::Navigate,
            iced::widget::Space::with_width(0).into(),
        ),
        layout::workspace(surface::scroll(
            column![identity_card(app), import_section(app), sections_grid(app),]
                .spacing(space::LG)
                .width(Length::Fill),
        )),
    )
}

/// Carte d'identité : nom, accroche, contacts et complétion, sans emplacement photo.
fn identity_card(app: &App) -> Element<'_, Message> {
    let profile = &app.data.profile;
    let personal = &profile.personal;
    let name = format!("{} {}", personal.first_name, personal.last_name)
        .trim()
        .to_owned();
    let score = completion_score(profile);

    container(
        row![
            column![
                typo::title(if name.is_empty() {
                    "Profil à compléter".to_owned()
                } else {
                    name
                }),
                typo::toned(
                    format::or_else(
                        personal.headline.as_deref(),
                        "Titre professionnel non renseigné",
                    ),
                    Tone::Accent,
                ),
                contact_chips(personal),
            ]
            .spacing(space::SM)
            .align_x(Alignment::Start),
            layout::spacer(),
            completion_panel(score),
        ]
        .spacing(space::XL)
        .align_y(Alignment::Center),
    )
    .padding(space::XXL)
    .width(Length::Fill)
    .style(styles::glass_card)
    .into()
}

/// Jetons de contact : e-mail, téléphone et ville.
fn contact_chips<'a, Message: 'a>(personal: &PersonalInfo) -> Element<'a, Message> {
    let values: Vec<String> = [
        Some(personal.email.as_str()).filter(|value| !value.is_empty()),
        personal.phone.as_deref(),
        personal.city.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(str::to_owned)
    .collect();

    let mut line = row![].spacing(space::SM);
    for value in values {
        line = line.push(chip(value));
    }
    line.into()
}

/// Jeton de contact : bordure fine sur fond ambiant translucide
/// (`rounded-full border bg-background/35`).
fn chip<'a, Message: 'a>(value: String) -> Element<'a, Message> {
    container(typo::caption(value))
        .padding([4.0, 10.0])
        .style(|theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(alpha(palette.canvas, 0.35))),
                border: Border {
                    color: palette.border,
                    width: stroke::HAIRLINE,
                    radius: radius::PILL.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}

/// Colonne de complétion : libellé, barre et pourcentage monospace.
fn completion_panel<'a, Message: 'a>(score: u8) -> Element<'a, Message> {
    column![
        typo::caption("Profil complété"),
        progress_bar(0.0..=1.0, f32::from(score) / 100.0)
            .height(6.0)
            .style(styles::progress(Tone::Accent)),
        typo::text_mono(format!("{score} %"), font::MICRO, font::MONO_SEMIBOLD),
    ]
    .spacing(space::XS)
    .width(Length::Fixed(COMPLETION_WIDTH))
    .into()
}

/// Composition éditoriale du profil : identité et compétences en premier,
/// parcours au centre, éléments complémentaires ensuite.
fn sections_grid(app: &App) -> Element<'_, Message> {
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
fn header_tile<'a, Message: 'a>(glyph: Icon) -> Element<'a, Message> {
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

/// Carte Import : choix du PDF, extraction et validation explicite.
fn import_section(app: &App) -> Element<'_, Message> {
    let file = app.profile_import_path.as_ref().map_or_else(
        || "Aucun CV sélectionné".to_owned(),
        |path| {
            path.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("cv.pdf")
                .to_owned()
        },
    );

    let source = container(
        row![
            header_tile(Icon::Document),
            column![
                typo::label("Document source"),
                typo::item(format::truncate(&file, 42)),
            ]
            .spacing(space::XS),
            layout::spacer(),
            controls::ghost("Parcourir", Some(Icon::Import)).on_press(Message::SelectProfilePdf),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .padding(space::MD)
    .width(Length::FillPortion(2))
    .style(styles::sunken);

    let action: Element<'_, Message> = if app.ai_is_running {
        container(state::running(
            "Extraction du profil",
            app.ai_elapsed_seconds,
            Message::CancelAi,
        ))
        .width(Length::FillPortion(2))
        .into()
    } else if app.extracted_profile.is_some() {
        column![
            badge::badge("Analyse terminée", Tone::Success),
            controls::primary("Vérifier les données", Some(Icon::ArrowRight))
                .on_press(Message::OpenDialog(Dialog::ProfileImport)),
        ]
        .spacing(space::SM)
        .align_x(Alignment::End)
        .into()
    } else {
        let mut analyze = controls::secondary("Analyser le CV", Some(Icon::Sparkles));
        if app.profile_import_path.is_some() {
            analyze = analyze.on_press(Message::ExtractProfile);
        }
        analyze.into()
    };

    container(
        row![
            row![
                header_tile(Icon::Import),
                column![
                    typo::section("Importer depuis un CV"),
                    typo::caption("L’IA prépare les données ; vous gardez le dernier mot."),
                ]
                .spacing(space::XS),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Center)
            .width(Length::FillPortion(2)),
            source,
            action,
        ]
        .spacing(space::XL)
        .align_y(Alignment::Center),
    )
    .padding(space::XL)
    .width(Length::Fill)
    .style(styles::glass_card)
    .into()
}

struct ImportProposal {
    label: String,
    value: String,
    meta: Option<String>,
    key: String,
}

struct ImportGroup {
    title: &'static str,
    kind: Icon,
    items: Vec<ImportProposal>,
}

/// Grand volet de validation, calqué sur le parcours de candilog-desktop.
pub fn import_review_drawer(app: &App) -> Element<'_, Message> {
    let Some(profile) = &app.extracted_profile else {
        return column![
            drawer_header(),
            container(state::empty_slot("Aucune donnée extraite à vérifier.")).center(Length::Fill),
        ]
        .height(Length::Fill)
        .into();
    };
    let groups = import_groups(profile);
    let total = groups.iter().map(|group| group.items.len()).sum::<usize>();
    let accepted = groups
        .iter()
        .flat_map(|group| &group.items)
        .filter(|item| !app.profile_import_excluded.contains(&item.key))
        .count();
    let breakdown = import_breakdown(&groups, &app.profile_import_excluded);
    let file = app.profile_import_path.as_ref().map_or_else(
        || "CV analysé".to_owned(),
        |path| {
            path.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("cv.pdf")
                .to_owned()
        },
    );

    let summary = container(
        column![
            container(icon::icon(Icon::Check, icon::LG, Ink::Toned(Tone::Success)))
                .center(Length::Fixed(42.0))
                .style(|theme: &Theme| import_toned_tile(theme, Tone::Success)),
            typo::section("CV analysé"),
            typo::caption(format::truncate(&file, 28)),
            surface::divider(),
            row![
                import_metric(accepted, "sélectionnées", Tone::Accent),
                import_metric(total.saturating_sub(accepted), "ignorées", Tone::Neutral),
            ]
            .spacing(space::SM),
            surface::divider(),
            typo::label("Sélection rapide"),
            controls::wide("Tout sélectionner", Some(Icon::Check))
                .on_press(Message::AcceptAllProfileImportItems),
            controls::wide("Tout ignorer", Some(Icon::Close))
                .on_press(Message::RejectAllProfileImportItems),
            surface::divider(),
            typo::label("Répartition"),
            breakdown,
            Space::with_height(Length::Fill),
            state::hint("Votre profil actuel reste prioritaire et les doublons sont écartés automatiquement."),
        ]
        .spacing(space::MD),
    )
    .width(Length::Fixed(220.0))
    .height(Length::Fill)
    .padding(space::XL)
    .style(styles::sunken);

    let mut suggestions = column![row![
        column![
            typo::section("Informations détectées"),
            typo::caption("Choisissez précisément ce qui rejoindra votre profil."),
        ]
        .spacing(space::XS),
        layout::spacer(),
        badge::count(total),
    ]
    .align_y(Alignment::Center),]
    .spacing(space::MD)
    .padding([0.0, space::XL]);
    for group in groups {
        suggestions = suggestions.push(import_group_card(group, &app.profile_import_excluded));
    }

    let mut apply = controls::primary("Ajouter au profil", Some(Icon::Check));
    if accepted > 0 {
        apply = apply.on_press(Message::ApplyExtractedProfile);
    }
    column![
        drawer_header(),
        row![
            summary,
            container(surface::scroll(suggestions).height(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill),
        ]
        .spacing(space::XL)
        .height(Length::Fill),
        surface::divider(),
        row![
            controls::ghost("Changer de CV", Some(Icon::Refresh))
                .on_press(Message::SelectProfilePdf),
            layout::spacer(),
            badge::badge(format!("{accepted} à ajouter"), Tone::Accent),
            apply,
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    ]
    .padding(space::XL)
    .spacing(space::XL)
    .height(Length::Fill)
    .into()
}

fn import_breakdown(
    groups: &[ImportGroup],
    excluded: &std::collections::HashSet<String>,
) -> Element<'static, Message> {
    let mut content = column![].spacing(space::SM);
    for group in groups {
        let selected = group
            .items
            .iter()
            .filter(|item| !excluded.contains(&item.key))
            .count();
        content = content.push(
            row![
                icon::icon(group.kind, icon::SM, Ink::Muted),
                typo::caption(group.title),
                layout::spacer(),
                typo::text_mono(
                    format!("{selected}/{}", group.items.len()),
                    font::MICRO,
                    font::MONO_SEMIBOLD,
                ),
            ]
            .spacing(space::SM)
            .align_y(Alignment::Center),
        );
    }
    content.into()
}

fn drawer_header() -> Element<'static, Message> {
    row![
        header_tile(Icon::Sparkles),
        column![
            typo::text_uppercase("IMPORT INTELLIGENT", font::MICRO, font::SEMIBOLD),
            typo::title("Vérifier les données du CV"),
            typo::caption("Rien n’est enregistré avant votre validation finale."),
        ]
        .spacing(space::XS),
        layout::spacer(),
        controls::icon_action(Icon::Close, "Fermer", Message::CloseDialog),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center)
    .into()
}

fn import_metric(value: usize, label: &'static str, tone: Tone) -> Element<'static, Message> {
    container(
        column![typo::metric(value.to_string()), typo::caption(label)]
            .spacing(space::XS)
            .align_x(Alignment::Start),
    )
    .padding(space::SM)
    .width(Length::Fill)
    .style(move |theme: &Theme| import_metric_style(theme, tone))
    .into()
}

fn import_group_card(
    group: ImportGroup,
    excluded: &std::collections::HashSet<String>,
) -> Element<'static, Message> {
    let count = group.items.len();
    let mut content = column![
        row![
            container(icon::icon(group.kind, icon::SM, Ink::Accent))
                .center(Length::Fixed(28.0))
                .style(|theme: &Theme| import_toned_tile(theme, Tone::Accent)),
            typo::section(group.title),
            layout::spacer(),
            badge::count(count),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
        surface::divider(),
    ]
    .spacing(space::SM);
    for item in group.items {
        let is_included = !excluded.contains(&item.key);
        let mut description = column![].spacing(space::XS);
        if !item.label.is_empty() {
            description = description.push(typo::label(item.label));
        }
        description = description.push(typo::item(format::truncate(&item.value, 72)));
        if let Some(meta) = item.meta {
            description = description.push(typo::caption(format::truncate(&meta, 78)));
        }
        content = content.push(
            container(
                row![
                    description.width(Length::Fill),
                    badge::badge(
                        if is_included {
                            "À importer"
                        } else {
                            "Ignoré"
                        },
                        if is_included {
                            Tone::Success
                        } else {
                            Tone::Neutral
                        },
                    ),
                    controls::icon_action(
                        if is_included { Icon::Close } else { Icon::Plus },
                        if is_included { "Ignorer" } else { "Inclure" },
                        Message::ToggleProfileImportItem(item.key),
                    ),
                ]
                .spacing(space::MD)
                .align_y(Alignment::Center),
            )
            .padding([space::SM, space::MD])
            .style(move |theme: &Theme| import_proposal_style(theme, is_included)),
        );
    }
    container(content)
        .padding(space::MD)
        .style(styles::panel)
        .into()
}

fn import_groups(profile: &Profile) -> Vec<ImportGroup> {
    let mut groups = Vec::new();
    let personal = [
        (
            "first_name",
            "Prénom",
            Some(profile.personal.first_name.as_str()),
        ),
        (
            "last_name",
            "Nom",
            Some(profile.personal.last_name.as_str()),
        ),
        ("email", "E-mail", Some(profile.personal.email.as_str())),
        ("phone", "Téléphone", profile.personal.phone.as_deref()),
        ("city", "Ville", profile.personal.city.as_deref()),
        (
            "headline",
            "Titre professionnel",
            profile.personal.headline.as_deref(),
        ),
        ("summary", "Résumé", profile.personal.summary.as_deref()),
        ("linkedin", "LinkedIn", profile.personal.linkedin.as_deref()),
        ("github", "GitHub", profile.personal.github.as_deref()),
        (
            "website",
            "Site / portfolio",
            profile.personal.website.as_deref(),
        ),
    ]
    .into_iter()
    .filter_map(|(field, label, value)| {
        value
            .filter(|value| !value.trim().is_empty())
            .map(|value| ImportProposal {
                label: label.into(),
                value: value.into(),
                meta: None,
                key: format!("personal.{field}:0"),
            })
    })
    .collect::<Vec<_>>();
    push_import_group(&mut groups, "Coordonnées & liens", Icon::Profile, personal);

    push_import_group(
        &mut groups,
        "Expériences",
        Icon::Building,
        profile
            .experiences
            .iter()
            .enumerate()
            .map(|(index, item)| ImportProposal {
                label: item.company.clone(),
                value: item.title.clone(),
                meta: Some(format_import_period(
                    &item.start_date,
                    item.end_date.as_deref(),
                    item.current,
                )),
                key: crate::app::profile_edit::import_item_key("experiences", index),
            })
            .collect(),
    );
    push_import_group(
        &mut groups,
        "Compétences",
        Icon::Sparkles,
        simple_import_items(
            "skills",
            profile.skills.iter().map(|item| item.name.clone()),
        ),
    );
    push_import_group(
        &mut groups,
        "Formations",
        Icon::Document,
        profile
            .education
            .iter()
            .enumerate()
            .map(|(index, item)| ImportProposal {
                label: item.school.clone(),
                value: item.degree.clone(),
                meta: None,
                key: crate::app::profile_edit::import_item_key("education", index),
            })
            .collect(),
    );
    push_import_group(
        &mut groups,
        "Langues",
        Icon::Network,
        profile
            .languages
            .iter()
            .enumerate()
            .map(|(index, item)| ImportProposal {
                label: item.level.clone(),
                value: item.name.clone(),
                meta: None,
                key: crate::app::profile_edit::import_item_key("languages", index),
            })
            .collect(),
    );
    push_import_group(
        &mut groups,
        "Projets",
        Icon::Document,
        simple_import_items(
            "projects",
            profile.projects.iter().map(|item| item.name.clone()),
        ),
    );
    push_import_group(
        &mut groups,
        "Certifications",
        Icon::Check,
        simple_import_items(
            "certifications",
            profile.certifications.iter().map(|item| item.name.clone()),
        ),
    );
    groups
}

fn simple_import_items(
    category: &'static str,
    values: impl Iterator<Item = String>,
) -> Vec<ImportProposal> {
    values
        .enumerate()
        .map(|(index, value)| ImportProposal {
            label: String::new(),
            value,
            meta: None,
            key: crate::app::profile_edit::import_item_key(category, index),
        })
        .collect()
}

fn push_import_group(
    groups: &mut Vec<ImportGroup>,
    title: &'static str,
    kind: Icon,
    items: Vec<ImportProposal>,
) {
    if !items.is_empty() {
        groups.push(ImportGroup { title, kind, items });
    }
}

fn format_import_period(start: &str, end: Option<&str>, current: bool) -> String {
    let end = if current {
        "Aujourd’hui"
    } else {
        end.unwrap_or("Date de fin non précisée")
    };
    if start.trim().is_empty() {
        end.to_owned()
    } else {
        format!("{start} — {end}")
    }
}

fn import_toned_tile(theme: &Theme, tone: Tone) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(tone.surface(&palette))),
        border: Border {
            color: tone.edge(&palette),
            width: stroke::HAIRLINE,
            radius: radius::CONTROL.into(),
        },
        ..container::Style::default()
    }
}

fn import_metric_style(theme: &Theme, tone: Tone) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(if tone == Tone::Neutral {
            palette.panel
        } else {
            tone.surface(&palette)
        })),
        border: Border {
            color: if tone == Tone::Neutral {
                palette.border
            } else {
                tone.edge(&palette)
            },
            width: stroke::HAIRLINE,
            radius: radius::CONTROL.into(),
        },
        ..container::Style::default()
    }
}

fn import_proposal_style(theme: &Theme, included: bool) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(if included {
            alpha(palette.success, if palette.is_dark { 0.055 } else { 0.035 })
        } else {
            alpha(palette.sunken, 0.55)
        })),
        border: Border {
            color: if included {
                alpha(palette.success, 0.22)
            } else {
                palette.border
            },
            width: stroke::HAIRLINE,
            radius: radius::CONTROL.into(),
        },
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::completion_score;
    use crate::shared::profile::{
        Certification, Education, Experience, Language, PersonalInfo, Profile, Project, Skill,
    };

    /// Profil dont exactement `count` sections sont complètes.
    fn profile_avec_sections(count: usize) -> Profile {
        let mut profile = Profile::default();
        if count >= 1 {
            profile.personal = PersonalInfo {
                first_name: "Alice".into(),
                last_name: "Dupont".into(),
                email: "alice@dupont.fr".into(),
                ..PersonalInfo::default()
            };
        }
        if count >= 2 {
            profile.experiences.push(Experience {
                title: "Développeuse".into(),
                company: "ACME".into(),
                ..Experience::default()
            });
        }
        if count >= 3 {
            profile.skills.push(Skill {
                name: "Rust".into(),
            });
        }
        if count >= 4 {
            profile.education.push(Education {
                degree: "Master".into(),
                school: "Université".into(),
                ..Education::default()
            });
        }
        if count >= 5 {
            profile.languages.push(Language {
                name: "Français".into(),
                level: "Natif".into(),
            });
        }
        if count >= 6 {
            profile.projects.push(Project {
                name: "Candilog".into(),
                ..Project::default()
            });
        }
        if count >= 7 {
            profile.certifications.push(Certification {
                name: "Certification".into(),
                ..Certification::default()
            });
        }
        profile
    }

    #[test]
    fn un_profil_vide_score_zero() {
        assert_eq!(completion_score(&Profile::default()), 0);
    }

    #[test]
    fn un_profil_complet_score_cent() {
        assert_eq!(completion_score(&profile_avec_sections(7)), 100);
    }

    #[test]
    fn un_profil_a_moitie_score_proche_de_cinquante() {
        // Pondération 1/7 : les scores entiers les plus proches de 50 sont
        // 3/7 ≈ 43 et 4/7 ≈ 57, arrondis au plus proche.
        assert_eq!(completion_score(&profile_avec_sections(3)), 43);
        assert_eq!(completion_score(&profile_avec_sections(4)), 57);
    }

    #[test]
    fn l_identite_n_est_complete_qu_avec_nom_prenom_et_email() {
        let mut profile = profile_avec_sections(0);
        profile.personal.email = "alice@dupont.fr".into();
        assert_eq!(completion_score(&profile), 0);

        profile.personal.first_name = "Alice".into();
        assert_eq!(completion_score(&profile), 0);

        profile.personal.last_name = "Dupont".into();
        assert_eq!(completion_score(&profile), 14);
    }
}
