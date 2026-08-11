//! Analyse d'un CV externe : carte « Nouvelle analyse » ou résultat ATS,
//! corps centré sur 960 px maximum.

use crate::app::{App, Message};
use crate::modules::ia::components::skill_list;
use crate::modules::ia::service::ImportedCvAnalysis;
use crate::ui::components::badge_score;
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::{self, Icon, Ink};
use crate::ui::components::score_gauge::{gauge, score_label, tone_pour_score};
use crate::ui::components::stat_card as cards;
use crate::ui::components::{badge, field, inspector, layout, list, state, surface, typo};
use crate::ui::theme::metrics::{radius, space};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{button, column, container, row, Space, Stack};
use iced::{Alignment, Background, Border, Element, Length, Shadow, Theme};

/// Largeur maximale du corps de l'écran (`max-w-[960px]`).
const BODY_MAX_WIDTH: f32 = 960.0;
/// Hauteur partagée de la zone de dépôt et de l'éditeur d'offre (`min-h-[210px]`).
const DROP_HEIGHT: f32 = 210.0;

/// Compétences présentes vs à renforcer depuis une analyse importée.
///
/// L'analyse porte déjà les listes `matched`/`missing` calculées par
/// `scoring::score_imported` : on ne recalcule rien, on déduplique seulement
/// (casse ignorée, ordre préservé).
fn skill_split(analysis: &ImportedCvAnalysis) -> (Vec<String>, Vec<String>) {
    (
        clean_skills(&analysis.score.matched),
        clean_skills(&analysis.score.missing),
    )
}

/// Déduplication insensible à la casse : premier exemplaire conservé.
fn clean_skills(skills: &[String]) -> Vec<String> {
    let mut seen = Vec::new();
    let mut cleaned = Vec::new();
    for skill in skills {
        let key = skill.to_lowercase();
        if !seen.contains(&key) {
            seen.push(key);
            cleaned.push(skill.clone());
        }
    }
    cleaned
}

/// Rend l'écran d'analyse d'un CV externe.
pub fn view(app: &App) -> Element<'_, Message> {
    let body: Element<'_, Message> = if app.ai_is_running {
        container(state::ai_progress(
            "Analyse du CV",
            app.ai_elapsed_seconds,
            Message::CancelAi,
        ))
        .height(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else if app.imported_cv_analysis.is_some() {
        result_card(app)
    } else {
        source_card(app)
    };

    layout::screen(
        header::page_header(
            Icon::Import,
            "Analysez un CV",
            "Confidentiel : tout reste sur cet appareil",
            Space::with_width(0).into(),
        ),
        layout::workspace(
            container(body)
                .max_width(BODY_MAX_WIDTH)
                .center_x(Length::Fill)
                .height(Length::Fill),
        ),
    )
}

/// Carte « Nouvelle analyse » : dépôt du PDF, offre à comparer, action.
fn source_card(app: &App) -> Element<'_, Message> {
    let content = column![
        surface::section_header(
            "Nouvelle analyse",
            badge::badge("Données confidentielles", Tone::Success),
        ),
        surface::divider(),
        row![drop_zone(app), offer_pane(app)]
            .spacing(space::LG)
            .width(Length::Fill),
        footer(),
    ]
    .spacing(space::LG);

    container(surface::scroll(container(content).padding(space::XL)))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(styles::glass_card)
        .into()
}

/// Zone de dépôt du PDF : tout le rectangle relance le sélecteur de fichier.
fn drop_zone(app: &App) -> Element<'_, Message> {
    let selected = app.import_pdf_path.is_some();
    let label = app.import_pdf_path.as_ref().map_or_else(
        || "Glissez un PDF ou choisissez".to_owned(),
        |path| {
            path.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("document.pdf")
                .to_owned()
        },
    );

    button(
        column![
            container(icon::icon(Icon::Document, 48.0, Ink::Muted))
                .width(72.0)
                .height(72.0)
                .center(Length::Fixed(72.0))
                .style(icon_tile),
            typo::body(label),
            row![
                icon::icon(Icon::Import, icon::SM, Ink::Accent),
                typo::toned("Choisir un PDF", Tone::Accent),
            ]
            .spacing(space::SM)
            .align_y(Alignment::Center),
        ]
        .spacing(space::MD)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(DROP_HEIGHT))
    .style(move |theme, status| {
        let palette = tokens(theme);
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: hovered
                .then_some(Background::Color(alpha(palette.accent, 0.045)))
                .or_else(|| selected.then_some(Background::Color(alpha(palette.accent, 0.04)))),
            text_color: palette.text,
            border: styles::drop_zone(selected)(theme).border,
            shadow: Shadow::default(),
        }
    })
    .on_press(Message::SelectImportPdf)
    .into()
}

/// Encoche du pictogramme de document (`bg-secondary/70 rounded-xl`).
fn icon_tile(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(alpha(palette.sunken, 0.70))),
        border: Border {
            radius: radius::CARD.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// Éditeur de l'offre à comparer, assorti à la zone de dépôt.
fn offer_pane(app: &App) -> Element<'_, Message> {
    column![
        field::editor(&app.import_offer_editor, "Collez l'offre cible…")
            .on_action(Message::ImportOfferEditorAction)
            .height(Length::Fixed(DROP_HEIGHT)),
        typo::caption("L'offre à comparer, si vous en avez une"),
    ]
    .spacing(space::XS)
    .width(Length::Fill)
    .into()
}

/// Pied de carte : rappel de confidentialité et action principale.
fn footer() -> Element<'static, Message> {
    container(
        row![
            typo::caption("Lecture locale du PDF, analyse via votre fournisseur IA"),
            layout::spacer(),
            controls::primary("Analyser le CV", Some(Icon::Sparkles))
                .on_press(Message::AnalyzeImportedCv),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
    )
    .padding([space::MD, space::LG])
    .width(Length::Fill)
    .style(styles::sunken)
    .into()
}

/// Résultat de l'analyse : badge de score, métriques, synthèse, conseils.
fn result_card(app: &App) -> Element<'_, Message> {
    let Some(result) = &app.imported_cv_analysis else {
        return state::empty_slot("Analyse indisponible.");
    };
    let score = result.analysis.score;
    let (present, to_strengthen) = skill_split(result);

    let header = container(
        row![
            typo::section("Résultat"),
            layout::spacer(),
            badge_score::score_badge(score),
            Stack::with_children(vec![
                gauge(score, 48.0),
                container(score_label(score))
                    .center_x(Length::Fixed(48.0))
                    .center_y(Length::Fixed(48.0))
                    .into(),
            ]),
        ]
        .spacing(space::MD)
        .align_y(Alignment::Center),
    )
    .padding([space::MD, space::XL])
    .width(Length::Fill)
    .style(move |theme: &Theme| container::Style {
        background: Some(Background::Color(alpha(tokens(theme).accent, 0.08))),
        ..container::Style::default()
    });

    let mut content = column![
        layout::columns([
            cards::metric_icon_tinted(
                "Compétences",
                format!("{} / 100", result.score.skills),
                tone_pour_score(result.score.skills),
                Icon::Check,
            ),
            cards::metric_icon_tinted(
                "Expérience",
                experience_label(result.score.experience),
                if result.score.experience == 0 {
                    Tone::Neutral
                } else {
                    tone_pour_score(result.score.experience)
                },
                Icon::Clock,
            ),
            cards::metric_icon_tinted(
                "Mots-clés ATS",
                format!("{} / 100", result.score.ats),
                tone_pour_score(result.score.ats),
                Icon::Target,
            ),
        ]),
        inspector::note("Synthèse", Some(result.analysis.recap.clone())),
        typo::label("Recommandations"),
        surface::divider(),
    ]
    .spacing(space::LG);

    if result.analysis.suggestions.is_empty() {
        content = content.push(state::empty_slot("Aucune recommandation complémentaire."));
    } else {
        for (index, suggestion) in result.analysis.suggestions.iter().enumerate() {
            content = content.push(list::row_static(
                container(
                    typo::text_mono(
                        format!("{:02}", index + 1),
                        font::MICRO,
                        font::MONO_SEMIBOLD,
                    )
                    .style(styles::secondary_text),
                )
                .width(space::XL),
                typo::body(suggestion.clone()),
                Space::with_width(0),
            ));
        }
    }

    content = content
        .push(surface::divider())
        .push(skill_list("Présentes dans le CV", &present, Tone::Success))
        .push(skill_list("À renforcer", &to_strengthen, Tone::Neutral));

    container(
        column![
            header,
            surface::divider(),
            surface::scroll(container(content).padding(space::XL)).height(Length::Fill),
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(styles::glass_card)
    .into()
}

/// Libellé de la métrique d'expérience d'un CV importé.
///
/// L'expérience n'est pas évaluable sur un CV importé (aucune date) :
/// `score_imported` la force à `0`. Un « 0 / 100 » serait trompeur, on affiche
/// donc « Non évaluée » tant que le sous-score vaut `0`.
fn experience_label(experience: u8) -> String {
    if experience == 0 {
        "Non évaluée".to_owned()
    } else {
        format!("{experience} / 100")
    }
}

#[cfg(test)]
mod tests {
    use super::{experience_label, skill_split};
    use crate::modules::ia::cv_model::{AtsAnalysis, GeneratedCv, MatchScore, ParsedOffer};
    use crate::modules::ia::service::ImportedCvAnalysis;

    fn analysis(matched: &[&str], missing: &[&str]) -> ImportedCvAnalysis {
        ImportedCvAnalysis {
            cv: GeneratedCv::default(),
            offer: ParsedOffer::default(),
            score: MatchScore {
                matched: matched.iter().map(|skill| (*skill).to_owned()).collect(),
                missing: missing.iter().map(|skill| (*skill).to_owned()).collect(),
                ..MatchScore::default()
            },
            analysis: AtsAnalysis::default(),
        }
    }

    fn owned(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn repartit_les_competences_suivant_le_modele() {
        let result = analysis(&["Rust", "Go"], &["SQL"]);
        assert_eq!(
            skill_split(&result),
            (owned(&["Rust", "Go"]), owned(&["SQL"]))
        );
    }

    #[test]
    fn les_doublons_sont_supprimes() {
        let result = analysis(&["Rust", "Rust", "Go"], &["SQL", "SQL"]);
        assert_eq!(
            skill_split(&result),
            (owned(&["Rust", "Go"]), owned(&["SQL"]))
        );
    }

    #[test]
    fn la_casse_est_ignoree() {
        let result = analysis(&["Rust", "rust"], &["Go", "GO"]);
        assert_eq!(skill_split(&result), (owned(&["Rust"]), owned(&["Go"])));
    }

    #[test]
    fn sans_competences_les_listes_sont_vides() {
        let result = analysis(&[], &[]);
        assert_eq!(skill_split(&result), (Vec::new(), Vec::new()));
    }

    #[test]
    fn lexperience_non_evaluee_naffiche_pas_un_zero_trompeur() {
        assert_eq!(experience_label(0), "Non évaluée");
    }

    #[test]
    fn lexperience_evaluee_conserve_le_format_sur_cent() {
        assert_eq!(experience_label(100), "100 / 100");
        assert_eq!(experience_label(42), "42 / 100");
    }
}
