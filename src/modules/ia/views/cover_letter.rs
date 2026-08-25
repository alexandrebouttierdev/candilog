//! Atelier de lettre de motivation : brief, page de document et itérations IA.

use crate::app::{App, Message};
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::tabs::Tab;
use crate::ui::components::{badge, field, layout, surface, tabs, typo};
use crate::ui::theme::metrics::{radius, space};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::font::Family;
use iced::widget::{column, container, row, text, Container, Space};
use iced::{Alignment, Background, Border, Element, Font, Length, Theme};

const SERIF: Font = Font {
    family: Family::Serif,
    ..Font::DEFAULT
};

const TONES: [&str; 3] = ["formal", "casual", "creative"];
const LENGTHS: [&str; 3] = ["short", "medium", "long"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LetterStatus {
    Vide,
    Brouillon,
    EnCours,
    Generee,
}

fn letter_status(output: &str, running: bool) -> LetterStatus {
    if running {
        if output.is_empty() {
            LetterStatus::EnCours
        } else {
            LetterStatus::Brouillon
        }
    } else if output.is_empty() {
        LetterStatus::Vide
    } else {
        LetterStatus::Generee
    }
}

fn char_count(value: &str) -> usize {
    value.chars().count()
}

/// Rend l'atelier complet de rédaction.
pub fn view(app: &App) -> Element<'_, Message> {
    let compact = app.layout().width < 1_400.0;
    let mut actions = row![].spacing(space::SM);
    if !app.letter_output.trim().is_empty() && !app.ai_is_running {
        let save: Element<'_, Message> = if compact {
            controls::icon_action(Icon::Save, "Enregistrer la lettre", Message::SaveLetter)
        } else {
            controls::secondary("Enregistrer", Some(Icon::Save))
                .on_press(Message::SaveLetter)
                .into()
        };
        actions = actions.push(save);
    }

    let panels: Element<'_, Message> = if compact {
        row![
            brief_panel(app).width(Length::FillPortion(4)),
            document_panel(app).width(Length::FillPortion(5)),
            iteration_panel(app).width(Length::FillPortion(4)),
        ]
        .spacing(space::LG)
        .height(Length::Fill)
        .into()
    } else {
        layout::columns([
            brief_panel(app).width(Length::Fixed(330.0)).into(),
            document_panel(app).width(Length::Fill).into(),
            iteration_panel(app).width(Length::Fixed(360.0)).into(),
        ])
    };

    layout::screen(
        header::page_header(
            Icon::Letter,
            "Lettre de motivation",
            "Rédigez une lettre ciblée",
            actions.into(),
        ),
        layout::workspace(
            column![context_strip(app), panels,]
                .spacing(space::LG)
                .height(Length::Fill),
        ),
    )
}

fn context_strip(app: &App) -> Element<'_, Message> {
    let status = letter_status(&app.letter_output, app.ai_is_running);
    container(
        row![
            indicator(status),
            column![
                typo::label(status.label()),
                typo::caption("Brief → rédaction → itérations → bibliothèque"),
            ]
            .spacing(space::XXS),
            layout::spacer(),
            badge::badge(
                match app.letter_tone.as_str() {
                    "casual" => "Ton naturel",
                    "creative" => "Ton créatif",
                    _ => "Ton formel",
                },
                Tone::Accent,
            ),
            badge::badge(
                match app.letter_length.as_str() {
                    "short" => "Format court",
                    "long" => "Format long",
                    _ => "Format moyen",
                },
                Tone::Neutral,
            ),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .padding([space::SM, space::XL])
    .width(Length::Fill)
    .style(styles::form_group)
    .into()
}

fn brief_panel(app: &App) -> Container<'_, Message> {
    let content = column![
        column![
            typo::meta_toned("BRIEF DE RÉDACTION", Tone::Accent),
            typo::section("Personnalisez votre candidature"),
            typo::caption("Les données du profil restent la source de vérité du document."),
        ]
        .spacing(space::XS),
        field::form_section(
            Icon::Target,
            "Cible",
            "Entreprise et poste visés",
            column![
                field::text_field(
                    "Entreprise",
                    &app.letter_company,
                    Message::LetterCompanyChanged,
                ),
                field::text_field(
                    "Poste ciblé",
                    &app.letter_job_title,
                    Message::LetterJobTitleChanged,
                ),
            ]
            .spacing(space::MD),
        ),
        field::form_section(
            Icon::Sparkles,
            "Style",
            "Ton et longueur du document",
            column![
                tabs::segmented(
                    [
                        Tab::new("Formel", app.letter_tone == "formal"),
                        Tab::new("Naturel", app.letter_tone == "casual"),
                        Tab::new("Créatif", app.letter_tone == "creative"),
                    ],
                    |index| Message::LetterToneChanged(TONES[index].to_owned()),
                ),
                tabs::segmented(
                    [
                        Tab::new("Courte", app.letter_length == "short"),
                        Tab::new("Moyenne", app.letter_length == "medium"),
                        Tab::new("Longue", app.letter_length == "long"),
                    ],
                    |index| Message::LetterLengthChanged(LENGTHS[index].to_owned()),
                ),
            ]
            .spacing(space::SM),
        ),
        field::form_section(
            Icon::Document,
            "Contexte",
            "Offre complète ou consignes spécifiques",
            column![
                row![
                    typo::label("Texte source"),
                    layout::spacer(),
                    controls::ghost("Coller", Some(Icon::Copy))
                        .on_press(Message::PasteLetterFromClipboard),
                ]
                .align_y(Alignment::Center),
                field::editor(
                    &app.letter_editor,
                    "Collez l'offre ou décrivez la candidature…",
                )
                .on_action(Message::LetterEditorAction)
                .height(Length::Fixed(210.0)),
            ]
            .spacing(space::SM),
        ),
    ]
    .spacing(space::LG);

    container(
        column![
            surface::scroll(container(content).padding(space::XL)).height(Length::Fill),
            surface::divider(),
            container(generate_button(app))
                .padding(space::XL)
                .width(Length::Fill),
        ]
        .height(Length::Fill),
    )
    .style(styles::glass_card)
    .height(Length::Fill)
}

fn generate_button(app: &App) -> Element<'_, Message> {
    if app.ai_is_running {
        controls::secondary("Arrêter la rédaction", Some(Icon::Stop))
            .on_press(Message::CancelAi)
            .width(Length::Fill)
            .height(40.0)
            .into()
    } else {
        controls::primary("Générer la lettre", Some(Icon::Sparkles))
            .on_press(Message::GenerateLetter)
            .width(Length::Fill)
            .height(40.0)
            .into()
    }
}

fn document_panel(app: &App) -> Container<'_, Message> {
    let status = letter_status(&app.letter_output, app.ai_is_running);
    let header = row![
        column![
            typo::section("Document"),
            typo::caption("Aperçu de la lettre prête à enregistrer"),
        ]
        .spacing(space::XXS),
        layout::spacer(),
        typo::caption(status.label()),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center);

    let page: Element<'_, Message> = if app.letter_output.is_empty() {
        placeholder(app)
    } else {
        letter_page(app)
    };

    container(
        column![
            container(header).padding([space::MD, space::XL]),
            surface::divider(),
            page,
            surface::divider(),
            container(document_footer(app))
                .padding([space::MD, space::XL])
                .width(Length::Fill),
        ]
        .height(Length::Fill),
    )
    .style(styles::glass_card)
    .height(Length::Fill)
}

fn letter_page(app: &App) -> Element<'_, Message> {
    let paper = container(
        column![
            typo::meta_toned("LETTRE DE MOTIVATION", Tone::Accent),
            text(if app.letter_job_title.trim().is_empty() {
                "Candidature".to_owned()
            } else {
                app.letter_job_title.clone()
            })
            .size(20.0)
            .font(font::SEMIBOLD),
            typo::caption(if app.letter_company.trim().is_empty() {
                "Entreprise".to_owned()
            } else {
                app.letter_company.clone()
            }),
            surface::divider(),
            text(app.letter_output.clone())
                .size(font::ITEM)
                .font(SERIF)
                .line_height(iced::widget::text::LineHeight::Absolute(25.0.into())),
        ]
        .spacing(space::LG),
    )
    .max_width(690.0)
    .padding([36.0, 44.0])
    .style(styles::document_paper);

    surface::scroll(container(paper).center_x(Length::Fill).padding(space::MAX))
        .height(Length::Fill)
        .into()
}

fn document_footer(app: &App) -> Element<'_, Message> {
    row![
        typo::caption("Contenu généré"),
        layout::spacer(),
        typo::text_mono(
            crate::ui::format::plural(char_count(&app.letter_output), "caractère", "caractères"),
            font::MICRO,
            font::MONO_REGULAR,
        ),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center)
    .into()
}

fn iteration_panel(app: &App) -> Container<'_, Message> {
    let history: Element<'_, Message> = if app.letter_chat_history.is_empty() {
        container(
            column![
                container(icon::icon(Icon::Sparkles, 28.0, Ink::Accent))
                    .width(54.0)
                    .height(54.0)
                    .center(Length::Fixed(54.0))
                    .style(iteration_icon_style),
                typo::label("Affinez sans repartir de zéro"),
                typo::caption(
                    "Demandez une introduction plus directe, un ton plus chaleureux ou une conclusion plus percutante.",
                ),
            ]
            .spacing(space::SM)
            .align_x(Alignment::Center),
        )
        .padding([space::MAX, space::XL])
        .center_x(Length::Fill)
        .into()
    } else {
        let mut items = column![].spacing(space::SM);
        for message in app
            .letter_chat_history
            .iter()
            .filter(|message| message.role == "user")
        {
            items = items.push(
                container(
                    column![
                        typo::meta_toned("CONSIGNE APPLIQUÉE", Tone::Accent),
                        typo::body(message.content.clone()),
                    ]
                    .spacing(space::XS),
                )
                .padding(space::MD)
                .width(Length::Fill)
                .style(styles::form_group),
            );
        }
        surface::scroll(items).height(Length::Fill).into()
    };

    let can_iterate = !app.letter_output.trim().is_empty()
        && !app.letter_iteration_instruction.trim().is_empty()
        && !app.ai_is_running;
    let mut action = controls::primary("Appliquer la consigne", Some(Icon::Sparkles))
        .width(Length::Fill)
        .height(38.0);
    if can_iterate {
        action = action.on_press(Message::IterateLetter);
    }

    container(
        column![
            container(
                column![
                    typo::meta_toned("ASSISTANT DE RÉÉCRITURE", Tone::Accent),
                    typo::section("Itération"),
                    typo::caption("Chaque consigne repart de la lettre visible."),
                ]
                .spacing(space::XS),
            )
            .padding(space::XL),
            surface::divider(),
            container(history).padding(space::XL).height(Length::Fill),
            surface::divider(),
            container(
                column![
                    field::input(
                        "Ex. Rendez la conclusion plus percutante…",
                        &app.letter_iteration_instruction,
                    )
                    .on_input(Message::LetterIterationChanged)
                    .width(Length::Fill),
                    action,
                ]
                .spacing(space::SM),
            )
            .padding(space::XL),
        ]
        .height(Length::Fill),
    )
    .style(styles::glass_card)
    .height(Length::Fill)
}

fn iteration_icon_style(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(alpha(palette.accent, 0.10))),
        border: Border {
            color: alpha(palette.accent, 0.22),
            width: 1.0,
            radius: radius::PILL.into(),
        },
        ..container::Style::default()
    }
}

fn placeholder(app: &App) -> Element<'_, Message> {
    container(
        column![
            container(icon::icon(Icon::Letter, 34.0, Ink::Accent))
                .width(66.0)
                .height(66.0)
                .center(Length::Fixed(66.0))
                .style(iteration_icon_style),
            typo::title(if app.letter_job_title.trim().is_empty() {
                "Votre prochaine lettre".to_owned()
            } else {
                app.letter_job_title.clone()
            }),
            typo::caption(if app.letter_company.trim().is_empty() {
                "Renseignez la cible et le contexte, puis lancez la rédaction.".to_owned()
            } else {
                format!(
                    "La lettre destinée à {} apparaîtra ici.",
                    app.letter_company
                )
            }),
        ]
        .spacing(space::MD)
        .align_x(Alignment::Center),
    )
    .padding(space::MAX)
    .center(Length::Fill)
    .into()
}

fn indicator(status: LetterStatus) -> Element<'static, Message> {
    container(Space::new(8.0, 8.0))
        .style(move |theme: &Theme| {
            let palette = tokens(theme);
            let color = match status {
                LetterStatus::EnCours | LetterStatus::Brouillon => palette.warning,
                LetterStatus::Generee => palette.success,
                LetterStatus::Vide => palette.text_muted,
            };
            container::Style {
                background: Some(Background::Color(color)),
                border: Border {
                    radius: radius::PILL.into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }
        })
        .into()
}

impl LetterStatus {
    const fn label(self) -> &'static str {
        match self {
            Self::Vide => "Prêt à rédiger",
            Self::Brouillon => "Brouillon en cours",
            Self::EnCours => "Rédaction en cours",
            Self::Generee => "Lettre prête",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{char_count, letter_status, LetterStatus, LENGTHS, TONES};

    #[test]
    fn les_options_de_style_sont_completes_et_distinctes() {
        assert_eq!(TONES.len(), 3);
        assert_eq!(LENGTHS.len(), 3);
        for options in [TONES, LENGTHS] {
            let unique: std::collections::BTreeSet<_> = options.iter().collect();
            assert_eq!(unique.len(), options.len());
        }
    }

    #[test]
    fn sans_contenu_ni_generation_le_document_est_vide() {
        assert_eq!(letter_status("", false), LetterStatus::Vide);
    }

    #[test]
    fn generation_sans_fragment_encore_est_en_cours() {
        assert_eq!(letter_status("", true), LetterStatus::EnCours);
    }

    #[test]
    fn generation_avec_fragments_est_un_brouillon() {
        assert_eq!(
            letter_status("Madame, Monsieur,", true),
            LetterStatus::Brouillon
        );
    }

    #[test]
    fn contenu_termine_est_genere() {
        assert_eq!(
            letter_status("Madame, Monsieur,", false),
            LetterStatus::Generee
        );
    }

    #[test]
    fn le_compteur_denombre_les_caracteres() {
        assert_eq!(char_count(""), 0);
        assert_eq!(char_count("café"), 4);
        assert_eq!(char_count("héllo 😀"), 7);
    }
}
