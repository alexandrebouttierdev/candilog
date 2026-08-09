//! Analyse d'un CV externe : étapes explicites, pas de zone de dépôt géante.

use crate::app::{App, Message};
use crate::ui::components::button as controls;
use crate::ui::components::icon::{self, Icon};
use crate::ui::components::{
    badge, field, inspector, layout, list, meter, state, surface, toolbar, typo,
};
use crate::ui::theme::metrics::space;
use crate::ui::theme::Tone;
use iced::widget::{column, container};
use iced::{Element, Length};

/// Étape courante du parcours d'import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Aucun document choisi.
    Selection,
    /// Document choisi, analyse possible.
    Ready,
    /// Analyse en cours.
    Running,
    /// Analyse terminée.
    Done,
}

impl Step {
    /// Détermine l'étape à partir de l'état applicatif.
    #[must_use]
    pub const fn resolve(has_file: bool, running: bool, has_result: bool) -> Self {
        if running {
            Self::Running
        } else if has_result {
            Self::Done
        } else if has_file {
            Self::Ready
        } else {
            Self::Selection
        }
    }

    /// Libellé de l'étape.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Selection => "Choisir un document",
            Self::Ready => "Prêt à analyser",
            Self::Running => "Analyse en cours",
            Self::Done => "Analyse terminée",
        }
    }

    /// Ton associé à l'étape.
    #[must_use]
    pub const fn tone(self) -> Tone {
        match self {
            Self::Selection => Tone::Neutral,
            Self::Ready => Tone::Info,
            Self::Running => Tone::Warning,
            Self::Done => Tone::Success,
        }
    }
}

/// Rend l'écran d'analyse d'un CV externe.
pub fn view(app: &App) -> Element<'_, Message> {
    let step = Step::resolve(
        app.import_pdf_path.is_some(),
        app.ai_is_running,
        app.imported_cv_analysis.is_some(),
    );

    let trailing = toolbar::group([if app.ai_is_running {
        controls::danger("Arrêter", Some(Icon::Stop))
            .on_press(Message::CancelAi)
            .into()
    } else {
        controls::primary("Analyser le CV", Some(Icon::Sparkles))
            .on_press(Message::AnalyzeImportedCv)
            .into()
    }]);

    layout::screen(
        toolbar::toolbar(
            "Analysez un CV",
            badge::badge(step.label(), step.tone()),
            trailing,
        ),
        layout::workspace(layout::columns([
            source_panel(app),
            result_panel(app, step),
        ])),
    )
}

fn source_panel(app: &App) -> Element<'_, Message> {
    let file = app.import_pdf_path.as_ref().map_or_else(
        || "Aucun document sélectionné".to_owned(),
        |path| {
            path.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("document.pdf")
                .to_owned()
        },
    );

    let mut panel = column![
        surface::section_header(
            "Document",
            controls::ghost("Choisir un PDF", Some(Icon::Import))
                .on_press(Message::SelectImportPdf),
        ),
        surface::divider(),
        list::row_static(
            icon::muted(Icon::Document),
            typo::body(file),
            typo::caption(if app.import_pdf_path.is_some() {
                "PDF"
            } else {
                "—"
            }),
        ),
        state::hint("Le PDF est lu localement ; rien n'est envoyé sans votre action."),
        surface::section_header("Offre à comparer", typo::caption("Optionnelle")),
        surface::divider(),
        field::editor(&app.import_offer_editor, "Collez l'offre cible…")
            .on_action(Message::ImportOfferEditorAction)
            .height(Length::Fixed(190.0)),
    ]
    .spacing(space::LG);

    if app.ai_is_running {
        panel = panel.push(state::running(
            "Lecture du PDF et analyse ATS",
            app.ai_elapsed_seconds,
            Message::CancelAi,
        ));
    }

    surface::panel(surface::scroll(panel).height(Length::Fill))
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .into()
}

fn result_panel(app: &App, step: Step) -> Element<'_, Message> {
    let body: Element<'_, Message> = match &app.imported_cv_analysis {
        None => state::empty(
            "Aucune analyse",
            match step {
                Step::Selection => "Choisissez un CV au format PDF pour lancer l'analyse.",
                _ => "Lancez l'analyse pour obtenir le score ATS et les suggestions.",
            },
        ),
        Some(result) => {
            let mut content = column![
                meter::ats(result.analysis.score),
                surface::divider(),
                inspector::note("Synthèse", Some(result.analysis.recap.clone())),
            ]
            .spacing(space::LG);
            if result.analysis.suggestions.is_empty() {
                content = content.push(state::empty_slot("Aucune suggestion complémentaire."));
            } else {
                content = content.push(typo::label("Suggestions"));
                content = content.push(surface::divider());
                for suggestion in &result.analysis.suggestions {
                    content = content.push(list::row_static(
                        icon::toned(Icon::Check, Tone::Success),
                        typo::body(suggestion.clone()),
                        iced::widget::Space::with_width(0),
                    ));
                }
            }
            surface::scroll(container(content).padding([space::LG, 0.0]))
                .height(Length::Fill)
                .into()
        }
    };

    surface::panel(
        column![
            surface::section_header("Résultat", badge::badge(step.label(), step.tone())),
            surface::divider(),
            body,
        ]
        .height(Length::Fill),
    )
    .width(Length::FillPortion(3))
    .height(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::Step;
    use crate::ui::theme::Tone;

    #[test]
    fn les_etapes_suivent_l_etat_applicatif() {
        assert_eq!(Step::resolve(false, false, false), Step::Selection);
        assert_eq!(Step::resolve(true, false, false), Step::Ready);
        assert_eq!(Step::resolve(true, true, false), Step::Running);
        assert_eq!(Step::resolve(true, false, true), Step::Done);
    }

    #[test]
    fn l_analyse_en_cours_prime_sur_un_resultat_precedent() {
        assert_eq!(Step::resolve(true, true, true), Step::Running);
    }

    #[test]
    fn chaque_etape_porte_un_libelle_et_un_ton_propres() {
        let steps = [Step::Selection, Step::Ready, Step::Running, Step::Done];
        for (index, step) in steps.iter().enumerate() {
            for other in &steps[index + 1..] {
                assert_ne!(step.label(), other.label());
                assert_ne!(step.tone(), other.tone());
            }
        }
        assert_eq!(Step::Done.tone(), Tone::Success);
    }
}
