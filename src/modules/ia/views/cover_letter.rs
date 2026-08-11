//! Lettre de motivation : brief à gauche, document au centre, itération à droite.

use crate::app::{App, Message};
use crate::navigation::Route;
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::Icon;
use crate::ui::components::tabs::Tab;
use crate::ui::components::{document, field, layout, state, surface, tabs, typo};
use crate::ui::theme::metrics::{radius, space};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::tokens;
use crate::ui::theme::typography as font;
use iced::font::Family;
use iced::widget::{column, container, row, text, Container, Space};
use iced::{Background, Border, Element, Font, Length, Theme};

/// Georgia : famille serif du corps de la lettre.
const SERIF: Font = Font {
    family: Family::Serif,
    ..Font::DEFAULT
};

/// Tons de rédaction proposés.
const TONES: [&str; 3] = ["formal", "casual", "creative"];
/// Longueurs de lettre proposées.
const LENGTHS: [&str; 3] = ["short", "medium", "long"];

/// Indicateur d'état du document de lettre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LetterStatus {
    /// Aucun contenu généré.
    Vide,
    /// Génération en cours, premiers fragments déjà présents.
    Brouillon,
    /// Génération en cours, aucun fragment encore reçu.
    EnCours,
    /// Contenu final disponible.
    Generee,
}

/// Indicateur d'état du document.
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

/// Nombre de caractères d'un texte.
fn char_count(text: &str) -> usize {
    text.chars().count()
}

/// Rend l'écran de génération de lettre.
pub fn view(app: &App) -> Element<'_, Message> {
    layout::screen(
        header::route_header(
            Icon::Letter,
            "Lettre de motivation",
            Route::LettreMotivation,
            Message::Navigate,
            Space::with_width(0).into(),
        ),
        layout::workspace(
            column![layout::columns([
                brief_panel(app).width(Length::FillPortion(290)).into(),
                document_panel(app).width(Length::FillPortion(1)).into(),
                chat_panel().width(Length::FillPortion(330)).into(),
            ]),]
            .spacing(space::LG)
            .height(Length::Fill),
        ),
    )
}

/// Colonne 1 : cible, style (segmented), contexte et génération.
fn brief_panel(app: &App) -> Container<'_, Message> {
    let content = column![
        surface::section_header("Cible", typo::caption("Entreprise et poste visés")),
        surface::divider(),
        field::text_field(
            "Entreprise",
            &app.letter_company,
            Message::LetterCompanyChanged
        ),
        field::text_field(
            "Poste ciblé",
            &app.letter_job_title,
            Message::LetterJobTitleChanged
        ),
        surface::section_header("Style", typo::caption("Ton et longueur")),
        surface::divider(),
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
        surface::section_header("Contexte", typo::caption("Offre ou consignes")),
        surface::divider(),
        field::editor(
            &app.letter_editor,
            "Collez l'offre ou décrivez la candidature…",
        )
        .on_action(Message::LetterEditorAction)
        .height(Length::Fixed(190.0)),
    ]
    .spacing(space::LG);

    container(
        column![
            surface::scroll(container(content).padding(space::XL)).height(Length::Fill),
            surface::divider(),
            container(generate_button(app))
                .padding([space::XL, space::XL])
                .width(Length::Fill),
        ]
        .height(Length::Fill),
    )
    .style(styles::glass_card)
    .height(Length::Fill)
}

/// Bouton de pied de brief : générer la lettre, ou arrêter pendant la rédaction.
fn generate_button(app: &App) -> Element<'_, Message> {
    if app.ai_is_running {
        controls::secondary("Arrêter", Some(Icon::Stop))
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

/// Colonne 2 : en-tête (indicateur + titre + état), page de lettre, compteur.
fn document_panel(app: &App) -> Container<'_, Message> {
    let status = letter_status(&app.letter_output, app.ai_is_running);

    let header = container(
        row![
            indicator(status),
            typo::section("Votre document"),
            layout::spacer(),
            typo::caption(status.label()),
        ]
        .spacing(space::MD)
        .align_y(iced::Alignment::Center),
    )
    .padding([space::MD, space::XL])
    .width(Length::Fill);

    let body: Element<'_, Message> = if app.letter_output.is_empty() {
        placeholder(app)
    } else {
        letter_page(app)
    };

    container(
        column![
            header,
            surface::divider(),
            body,
            surface::divider(),
            container(footer(app))
                .padding([space::MD, space::XL])
                .width(Length::Fill),
        ]
        .height(Length::Fill),
    )
    .style(styles::glass_card)
    .height(Length::Fill)
}

/// Point de 6 px : ambre pendant la rédaction, émeraude si la lettre existe.
fn indicator(status: LetterStatus) -> Element<'static, Message> {
    container(Space::new(6.0, 6.0))
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

/// Page de lettre : papier, texte serif à 13,5 px, interligne 28.
fn letter_page(app: &App) -> Element<'_, Message> {
    surface::scroll(
        container(
            text(app.letter_output.clone())
                .size(font::ITEM)
                .font(SERIF)
                .line_height(28.0)
                .style(|theme: &Theme| iced::widget::text::Style {
                    color: Some(tokens(theme).paper_ink),
                }),
        )
        .padding(space::XL)
        .width(Length::Fill)
        .style(move |theme: &Theme| {
            let palette = tokens(theme);
            container::Style {
                background: Some(Background::Color(palette.paper)),
                text_color: Some(palette.paper_ink),
                border: Border {
                    color: palette.paper_rule,
                    width: 1.0,
                    radius: radius::DOCUMENT.into(),
                },
                ..container::Style::default()
            }
        }),
    )
    .height(Length::Fill)
    .into()
}

/// Compteur de caractères, en pied de document.
fn footer(app: &App) -> Element<'_, Message> {
    row![
        typo::caption("Contenu"),
        layout::spacer(),
        typo::text_mono(
            crate::ui::format::plural(char_count(&app.letter_output), "caractère", "caractères"),
            font::MICRO,
            font::MONO_REGULAR,
        ),
    ]
    .spacing(space::MD)
    .align_y(iced::Alignment::Center)
    .into()
}

/// Colonne 3 : itération en lecture seule pour ce jalon.
fn chat_panel() -> Container<'static, Message> {
    container(
        container(
            column![
                surface::section_header("Itération", typo::caption("Consignes de réécriture")),
                surface::divider(),
                state::empty_slot("L'itération arrive au prochain jalon."),
                layout::spacer(),
                field::input("Écrivez une consigne…", "").width(Length::Fill),
            ]
            .spacing(space::LG)
            .height(Length::Fill),
        )
        .padding(space::XL),
    )
    .style(styles::glass_card)
    .height(Length::Fill)
}

/// État vide de la page : objet, entreprise et invitation à générer.
fn placeholder(app: &App) -> Element<'_, Message> {
    container(
        column![
            document::heading(if app.letter_job_title.trim().is_empty() {
                "Objet de la lettre".to_owned()
            } else {
                app.letter_job_title.clone()
            }),
            document::subheading(if app.letter_company.trim().is_empty() {
                "Entreprise".to_owned()
            } else {
                app.letter_company.clone()
            }),
            iced::widget::Space::with_height(space::MAX),
            document::body_muted(
                "La lettre s'écrira ici au fil de la génération, phrase après phrase.",
            ),
        ]
        .spacing(space::SM)
        .width(Length::Fill),
    )
    .padding(space::XL)
    .center_x(Length::Fill)
    .into()
}

impl LetterStatus {
    /// Libellé affiché à côté de l'indicateur.
    fn label(self) -> &'static str {
        match self {
            LetterStatus::Vide => "En attente de génération",
            LetterStatus::Brouillon => "Brouillon en cours",
            LetterStatus::EnCours => "Rédaction en cours",
            LetterStatus::Generee => "Lettre générée",
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
