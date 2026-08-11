//! Écran Profil : identité, complétion et sections en cartes sur une seule
//! page défilante, dans l'esprit candilog-desktop.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::profil::components as rows;
use crate::navigation::Route;
use crate::shared::profile::{PersonalInfo, Profile};
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::{avatar, badge, inspector, layout, list, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{column, container, progress_bar, row, text};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Largeur maximale de la carte d'identité (`max-w-[980px]`).
const IDENTITY_MAX_WIDTH: f32 = 980.0;
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
        !profile.experiences.is_empty(),
        !profile.skills.is_empty(),
        !profile.education.is_empty(),
        !profile.languages.is_empty(),
        !profile.projects.is_empty(),
        !profile.certifications.is_empty(),
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
            controls::ghost("Modifier", Some(Icon::Edit))
                .on_press(Message::OpenDialog(Dialog::Profil))
                .into(),
        ),
        layout::workspace(surface::scroll(
            column![
                container(identity_card(app))
                    .max_width(IDENTITY_MAX_WIDTH)
                    .center_x(Length::Fill),
                sections_grid(app),
            ]
            .spacing(space::LG)
            .width(Length::Fill),
        )),
    )
}

/// Carte d'identité : avatar, nom, accroche, jetons de contact et barre de
/// complétion.
fn identity_card(app: &App) -> Element<'_, Message> {
    let profile = &app.data.profile;
    let personal = &profile.personal;
    let name = format!("{} {}", personal.first_name, personal.last_name)
        .trim()
        .to_owned();
    let score = completion_score(profile);

    container(
        row![
            avatar::avatar(avatar::initials_of(&name), 76.0, Tone::Accent),
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
        row![
            container(identity_section(app)).width(Length::FillPortion(3)),
            container(skills_section(app)).width(Length::FillPortion(2)),
        ]
        .spacing(space::LG),
        row![
            container(experiences_section(app)).width(Length::FillPortion(1)),
            container(formations_section(app)).width(Length::FillPortion(1)),
        ]
        .spacing(space::LG),
        row![
            container(languages_section(app)).width(Length::FillPortion(1)),
            container(projects_section(app)).width(Length::FillPortion(1)),
            container(certifications_section(app)).width(Length::FillPortion(1)),
        ]
        .spacing(space::LG),
        import_section(app),
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
    pencil: bool,
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
    if pencil {
        header = header.push(controls::icon_action(
            Icon::Edit,
            "Modifier",
            Message::OpenDialog(Dialog::Profil),
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
        true,
        column![
            inspector::group(
                "Coordonnées",
                [
                    inspector::property("E-mail", format::or_dash(Some(&personal.email))),
                    inspector::property("Téléphone", format::or_dash(personal.phone.as_deref())),
                    inspector::property("Ville", format::or_dash(personal.city.as_deref())),
                ],
            ),
            inspector::group(
                "Présence en ligne",
                [
                    inspector::property("LinkedIn", format::or_dash(personal.linkedin.as_deref())),
                    inspector::property("GitHub", format::or_dash(personal.github.as_deref())),
                    inspector::property("Site web", format::or_dash(personal.website.as_deref())),
                ],
            ),
            inspector::note("Résumé", personal.summary.clone()),
        ]
        .spacing(space::XXL)
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
        true,
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
        true,
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
        true,
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
        true,
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
        true,
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
        true,
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

    section_card(
        Icon::Import,
        "Importer un CV",
        None,
        false,
        container(body).into(),
    )
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
            profile.experiences.push(Experience::default());
        }
        if count >= 3 {
            profile.skills.push(Skill::default());
        }
        if count >= 4 {
            profile.education.push(Education::default());
        }
        if count >= 5 {
            profile.languages.push(Language::default());
        }
        if count >= 6 {
            profile.projects.push(Project::default());
        }
        if count >= 7 {
            profile.certifications.push(Certification::default());
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
