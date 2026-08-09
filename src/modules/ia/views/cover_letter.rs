//! Lettre de motivation : configuration à gauche, feuille de rédaction à droite.

use crate::app::{App, Message};
use crate::ui::components::button as controls;
use crate::ui::components::icon::Icon;
use crate::ui::components::{badge, document, field, layout, state, surface, toolbar, typo};
use crate::ui::theme::metrics::space;
use crate::ui::theme::Tone;
use iced::widget::{column, container};
use iced::{Element, Length};

/// Tons de rédaction proposés.
const TONES: [&str; 3] = ["formal", "casual", "creative"];
/// Longueurs de lettre proposées.
const LENGTHS: [&str; 3] = ["short", "medium", "long"];

/// Rend l'écran de génération de lettre.
pub fn view(app: &App) -> Element<'_, Message> {
    let status = if app.ai_is_running {
        badge::badge("Rédaction en cours", Tone::Warning)
    } else if app.letter_output.is_empty() {
        badge::badge("Prêt", Tone::Neutral)
    } else {
        badge::badge("Lettre générée", Tone::Success)
    };

    let trailing = toolbar::group([if app.ai_is_running {
        controls::danger("Arrêter", Some(Icon::Stop))
            .on_press(Message::CancelAi)
            .into()
    } else {
        controls::primary("Générer la lettre", Some(Icon::Sparkles))
            .on_press(Message::GenerateLetter)
            .into()
    }]);

    layout::screen(
        toolbar::toolbar("Lettre de motivation", status, trailing),
        layout::split_portions(5, configuration(app), 6, editor(app)),
    )
}

fn configuration(app: &App) -> Element<'_, Message> {
    let mut panel = column![
        surface::section_header("Cible", typo::caption("Entreprise et poste visés")),
        surface::divider(),
        field::form_row([
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
        ]),
        surface::section_header("Contexte", typo::caption("Offre ou consignes")),
        surface::divider(),
        field::editor(
            &app.letter_editor,
            "Collez l'offre ou décrivez la candidature…",
        )
        .on_action(Message::LetterEditorAction)
        .height(Length::Fixed(190.0)),
        surface::section_header("Style", typo::caption("Ton et longueur")),
        surface::divider(),
        field::form_row([
            field::labeled(
                "Ton",
                field::select(
                    TONES.map(str::to_owned).to_vec(),
                    Some(app.letter_tone.clone()),
                    Message::LetterToneChanged,
                )
                .width(Length::Fill),
            ),
            field::labeled(
                "Longueur",
                field::select(
                    LENGTHS.map(str::to_owned).to_vec(),
                    Some(app.letter_length.clone()),
                    Message::LetterLengthChanged,
                )
                .width(Length::Fill),
            ),
        ]),
    ]
    .spacing(space::LG);

    if app.ai_is_running {
        panel = panel.push(state::running(
            "Rédaction en direct",
            app.ai_elapsed_seconds,
            Message::CancelAi,
        ));
    }

    surface::scroll(container(panel).padding(space::XL))
        .height(Length::Fill)
        .into()
}

fn editor(app: &App) -> Element<'_, Message> {
    let written = app.letter_output.chars().count();
    let bar = document::workbench_bar(
        "Lettre",
        typo::caption(if written == 0 {
            "En attente de génération".to_owned()
        } else {
            crate::ui::format::plural(written, "caractère", "caractères")
        }),
    );

    let page: Element<'_, Message> = if app.letter_output.is_empty() {
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
        .into()
    } else {
        column![
            document::heading(if app.letter_job_title.trim().is_empty() {
                "Candidature".to_owned()
            } else {
                app.letter_job_title.clone()
            }),
            document::subheading(app.letter_company.clone()),
            iced::widget::Space::with_height(space::LG),
            document::body(app.letter_output.clone()),
        ]
        .spacing(space::SM)
        .into()
    };

    column![
        container(bar).width(Length::Fill),
        surface::divider(),
        document::workspace(document::page(app.document_width, page)),
    ]
    .height(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::{LENGTHS, TONES};

    #[test]
    fn les_options_de_style_sont_completes_et_distinctes() {
        assert_eq!(TONES.len(), 3);
        assert_eq!(LENGTHS.len(), 3);
        for options in [TONES, LENGTHS] {
            let unique: std::collections::BTreeSet<_> = options.iter().collect();
            assert_eq!(unique.len(), options.len());
        }
    }
}
