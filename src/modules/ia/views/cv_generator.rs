//! CV Generator : workflow, jauge de score et atelier d'analyse à gauche,
//! document A4 posé à droite.

use crate::app::state::RecommendationStatus;
use crate::app::{App, Message};
use crate::modules::entreprises::model::Entreprise;
use crate::modules::ia::components::{recommendation, skill_list};
use crate::modules::ia::cv_model::{CvGeneration, MatchScore, OfferAnalysis};
use crate::navigation::Route;
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::{self, Icon};
use crate::ui::components::score_gauge::{gauge, score_label, tone_pour_score};
use crate::ui::components::workflow::{steps, StepState, WorkflowStep};
use crate::ui::components::{badge, document, field, layout, meter, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::{radius, size, space, stroke};
use crate::ui::theme::styles;
use crate::ui::theme::tokens::{alpha, tokens};
use crate::ui::theme::typography as font;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, Stack};
use iced::{Alignment, Background, Border, Element, Length, Theme};

/// Rend l'écran du générateur de CV.
pub fn view(app: &App) -> Element<'_, Message> {
    layout::screen(
        header::route_header(
            Icon::Sparkles,
            "Générateur de CV",
            Route::CvGenerator,
            Message::Navigate,
            actions(app),
        ),
        layout::workspace(
            column![
                overview(app),
                layout::columns([workbench(app), preview(app)]),
            ]
            .spacing(space::LG)
            .height(Length::Fill),
        ),
    )
}

/// En-tête : badge « IA », entreprise détectée puis actions révélées par l'avancée.
fn actions(app: &App) -> Element<'_, Message> {
    let mut actions = row![badge::badge("IA", Tone::Accent)]
        .spacing(space::SM)
        .align_y(Alignment::Center);
    if let Some(company) = detected_company(&app.offer_editor.text(), &app.data.entreprises) {
        actions = actions.push(company_pill(company));
    }
    if app.ai_is_running {
        actions = actions
            .push(controls::ghost("Tout annuler", Some(Icon::Stop)).on_press(Message::CancelAi));
    }
    if app.offer_analysis.is_some() {
        actions = actions.push(
            controls::secondary("Régénérer", Some(Icon::Refresh)).on_press(Message::GenerateCv),
        );
    }
    if app.cv_generation.is_some() {
        actions = actions.push(
            controls::primary("Exporter PDF", Some(Icon::Download))
                .on_press(Message::ExportGeneratedCvPdf),
        );
    }
    actions.into()
}

/// Pastille « entreprise détectée » : bâtiment et nom, en ton neutre.
fn company_pill<'a, Message: 'a>(company: String) -> Element<'a, Message> {
    container(
        row![
            icon::icon(Icon::Building, icon::SM, icon::Ink::Muted),
            typo::caption(company),
        ]
        .spacing(space::XS)
        .align_y(Alignment::Center),
    )
    .height(size::TAG)
    .padding([space::XXS, space::SM])
    .align_y(Alignment::Center)
    .style(styles::toned(Tone::Neutral))
    .into()
}

/// Détecte une entreprise connue citée dans le texte de l'offre.
fn detected_company(offer_text: &str, companies: &[Entreprise]) -> Option<String> {
    let needle = offer_text.to_lowercase();
    companies
        .iter()
        .map(|company| company.nom.trim())
        .filter(|nom| !nom.is_empty())
        // Les noms courts (ex. « IT », « Dev » ou « SA ») ne doivent pas être
        // reconnus au milieu d'un autre mot. Les noms composés et précis sont
        // testés en premier pour éviter qu'un sous-nom masque la bonne société.
        .filter(|nom| nom.chars().count() >= 3)
        .filter_map(|nom| {
            whole_phrase_position(&needle, &nom.to_lowercase()).and_then(|position| {
                company_context_score(&needle, &nom.to_lowercase(), position)
                    .map(|score| (score, position, nom))
            })
        })
        .max_by_key(|(score, position, nom)| {
            (*score, std::cmp::Reverse(*position), nom.chars().count())
        })
        .map(|(_, _, nom)| nom.to_owned())
}

fn whole_phrase_position(text: &str, phrase: &str) -> Option<usize> {
    text.match_indices(phrase).find_map(|(start, _)| {
        let end = start + phrase.len();
        let before = text[..start].chars().next_back();
        let after = text[end..].chars().next();
        (!before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric))
            .then_some(start)
    })
}

/// Évite de prendre une entreprise citée comme exemple, client ou concurrent.
/// Une détection est retenue uniquement si le nom est dans un contexte explicite
/// (`chez`, `rejoignez`, `notre client`...) ou s'il forme un en-tête proche du début.
fn company_context_score(text: &str, phrase: &str, position: usize) -> Option<u8> {
    let before_start = position.saturating_sub(96);
    let before = format!(" {}", &text[before_start..position]);
    let line_start = text[..position].rfind('\n').map_or(0, |index| index + 1);
    let line_end = text[position..]
        .find('\n')
        .map_or(text.len(), |index| position + index);
    let line = text[line_start..line_end].trim();
    let explicit_markers = [
        " chez ",
        " rejoignez ",
        " au sein de ",
        " notre client ",
        " l'entreprise ",
        " pour ",
        " entreprise ",
    ];
    if explicit_markers
        .iter()
        .any(|marker| before.contains(marker))
    {
        return Some(3);
    }
    if position < 400 && line.to_lowercase().starts_with(phrase) && line.chars().count() <= 120 {
        return Some(2);
    }
    None
}

/// Bandeau sous l'en-tête : étapes du parcours à gauche, jauge ATS à droite.
fn overview(app: &App) -> Element<'_, Message> {
    let analysed = app.offer_analysis.is_some();
    let generated = app.cv_generation.is_some();
    let items = [
        WorkflowStep::new(
            "Analyser",
            "L'offre",
            if analysed {
                StepState::Done
            } else {
                StepState::Active
            },
        ),
        WorkflowStep::new(
            "Améliorer",
            "Le CV",
            if generated {
                StepState::Done
            } else if analysed {
                StepState::Active
            } else {
                StepState::Pending
            },
        ),
        WorkflowStep::new(
            "Exporter",
            "Le PDF",
            if generated {
                StepState::Active
            } else {
                StepState::Pending
            },
        ),
    ];
    let score = app
        .offer_analysis
        .as_ref()
        .map_or(0, |analysis| analysis.score.total);

    row![
        steps(&items),
        layout::spacer(),
        column![
            Stack::with_children(vec![
                gauge(score, 48.0),
                container(score_label(score))
                    .center_x(Length::Fixed(48.0))
                    .center_y(Length::Fixed(48.0))
                    .into(),
            ]),
            typo::caption("Score ATS"),
        ]
        .spacing(space::XS)
        .align_x(Alignment::Center),
    ]
    .spacing(space::LG)
    .align_y(Alignment::Center)
    .into()
}

/// Atelier d'analyse : offre à coller, puis correspondance et suggestions.
fn workbench(app: &App) -> Element<'_, Message> {
    let panel = match &app.offer_analysis {
        None => offer_panel(app),
        Some(analysis) => analysis_panel(app, analysis),
    };
    surface::scroll(container(panel).padding(space::XL))
        .height(Length::Fill)
        .into()
}

/// Étape 1 : carte info d'aide à gauche, offre à analyser à droite (grille 0.72fr/1.28fr).
fn offer_panel(app: &App) -> Element<'_, Message> {
    row![
        container(info_panel(app))
            .width(Length::FillPortion(72))
            .padding(space::XL)
            .style(primary_info_style),
        container(offer_content(app))
            .width(Length::FillPortion(128))
            .padding(space::XL)
            .style(styles::glass_card),
    ]
    .spacing(space::LG)
    .into()
}

/// Carte info de l'étape 1 : ce que l'analyse produira, et contexte de l'offre.
fn info_panel(app: &App) -> Element<'_, Message> {
    let mut content =
        column![
        surface::section_header("Analyse de l'offre", badge::badge("3 étapes", Tone::Neutral)),
        surface::divider(),
        info_row(
            Icon::Target,
            "Compétences",
            "Les compétences de l'offre sont comparées à votre profil, présentes ou manquantes.",
        ),
        info_row(
            Icon::Chart,
            "Score ATS",
            "Une jauge 0–100 mesure la correspondance entre l'offre et votre profil.",
        ),
        info_row(
            Icon::Sparkles,
            "Suggestions",
            "Des recommandations ciblées préparent l'amélioration puis l'export du CV.",
        ),
    ]
        .spacing(space::LG);

    if let Some(company) = detected_company(&app.offer_editor.text(), &app.data.entreprises) {
        content = content.push(surface::divider()).push(
            row![
                icon::icon(Icon::Building, icon::MD, icon::Ink::Toned(Tone::Accent)),
                column![typo::label("Entreprise détectée"), typo::body(company),]
                    .spacing(1)
                    .align_x(Alignment::Start),
            ]
            .spacing(space::MD)
            .align_y(Alignment::Start),
        );
    }

    content.into()
}

/// Ligne d'information d'une carte : icône teintée, intitulé et description.
fn info_row<'a>(glyph: Icon, title: &'a str, detail: &'a str) -> Element<'a, Message> {
    row![
        icon::icon(glyph, icon::MD, icon::Ink::Toned(Tone::Accent)),
        column![typo::label(title), typo::caption(detail),]
            .spacing(1)
            .align_x(Alignment::Start),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Start)
    .into()
}

/// Carte offre de l'étape 1 : en-tête, éditeur et pied d'action.
fn offer_content(app: &App) -> Element<'_, Message> {
    let footer: Element<'_, Message> = if app.ai_is_running {
        state::running(
            "Analyse de l'offre",
            app.ai_elapsed_seconds,
            Message::CancelAi,
        )
    } else {
        container(
            row![
                typo::text_mono(
                    // `Content::text()` renvoie un saut de ligne terminal : sans le retirer,
                    // un éditeur vide affichait « 1 caractères » — faux compte et accord
                    // fautif. `format::plural` fait l'accord, comme le fait déjà l'écran
                    // voisin de la lettre de motivation.
                    format::plural(
                        app.offer_editor
                            .text()
                            .trim_end_matches('\n')
                            .chars()
                            .count(),
                        "caractère",
                        "caractères",
                    ),
                    font::MICRO,
                    font::MONO_REGULAR,
                ),
                layout::spacer(),
                controls::primary("Analyser l'offre", Some(Icon::Target))
                    .on_press(Message::AnalyzeOffer),
            ]
            .spacing(space::SM)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .into()
    };
    column![
        surface::section_header(
            "Offre ciblée",
            typo::caption("Collez le texte complet de l'annonce"),
        ),
        surface::divider(),
        row![
            typo::label("Texte intégral de l'offre"),
            layout::spacer(),
            controls::secondary("Coller depuis le presse-papiers", Some(Icon::Copy))
                .on_press(Message::PasteOfferFromClipboard),
        ]
        .align_y(Alignment::Center),
        field::editor(&app.offer_editor, "Collez ici le texte complet de l'offre…")
            .on_action(Message::OfferEditorAction)
            // Zone volontairement haute : une offre longue reste lisible sans
            // donner l'impression d'être tronquée. L'éditeur conserve son
            // défilement interne au-delà de cette hauteur.
            .height(Length::Fixed(480.0)),
        footer,
    ]
    .spacing(space::LG)
    .into()
}

/// Carte info de l'étape 1 : fond d'accent discret et filet doux
/// (`bg-primary/3.5`, `border-primary/15`), rayon de carte standard.
fn primary_info_style(theme: &Theme) -> container::Style {
    let palette = tokens(theme);
    container::Style {
        background: Some(Background::Color(alpha(palette.accent, 0.035))),
        text_color: Some(palette.text),
        border: Border {
            color: alpha(palette.accent, 0.15),
            width: stroke::HAIRLINE,
            radius: radius::CARD.into(),
        },
        ..container::Style::default()
    }
}

/// Étape 2 : correspondance profil × offre, suggestions ATS et version.
fn analysis_panel<'a>(app: &'a App, analysis: &'a OfferAnalysis) -> Element<'a, Message> {
    let present = present_skills(&analysis.score);
    let missing = missing_skills(&analysis.score);

    let mut content = column![
        surface::section_header(
            "Correspondance",
            badge::badge(
                format!("{} / 100", analysis.score.total),
                tone_pour_score(analysis.score.total),
            ),
        ),
        surface::divider(),
        skill_list("Présentes dans votre profil", &present, Tone::Success),
        skill_list("Manquantes dans votre profil", &missing, Tone::Neutral),
    ]
    .spacing(space::LG);

    if let Some(generation) = &app.cv_generation {
        content = content
            .push(surface::divider())
            .push(surface::section_header(
                "Suggestions ATS",
                badge::count(generation.analysis.recommandations.len()),
            ))
            .push(meter::ats(generation.analysis.score));
        if generation.analysis.recommandations.is_empty() {
            content = content.push(state::empty_slot("Aucune suggestion à appliquer."));
        }
        for (index, item) in generation.analysis.recommandations.iter().enumerate() {
            let status = app
                .recommendation_states
                .get(index)
                .copied()
                .unwrap_or(RecommendationStatus::Pending);
            content = content.push(recommendation(
                item.section.clone(),
                item.impact,
                item.texte_original.clone(),
                item.texte_propose.clone(),
                status,
                Message::AcceptRecommendation(index),
                Message::RejectRecommendation(index),
            ));
        }
        content = content
            .push(surface::divider())
            .push(surface::section_header(
                "Enregistrer la version",
                controls::secondary("Enregistrer", Some(Icon::Save))
                    .on_press(Message::SaveGeneratedCv),
            ))
            .push(
                field::input("Nom de la version", &app.cv_version_name)
                    .on_input(Message::CvVersionNameChanged)
                    .width(Length::Fill),
            );
    }

    container(content)
        .padding(space::XL)
        .style(styles::glass_card)
        .into()
}

/// Action offerte en pied du panneau d'analyse, entre analyse et génération.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PanelFooterState {
    /// Rien à proposer : pas d'analyse, ou une génération est déjà affichée.
    None,
    /// Analyse terminée sans génération : proposer « Améliorer le CV ».
    ProposeGeneration,
    /// Génération en cours : montrer la progression et l'arrêt.
    Generating,
}

/// Décide du pied du panneau d'analyse.
///
/// Une génération déjà affichée prime : pendant une régénération, les suggestions
/// restent visibles et le pied ne reprend pas la main.
#[must_use]
fn panel_footer_state(analysed: bool, running: bool, generated: bool) -> PanelFooterState {
    if !analysed || generated {
        PanelFooterState::None
    } else if running {
        PanelFooterState::Generating
    } else {
        PanelFooterState::ProposeGeneration
    }
}

/// Aperçu du document A4, posé sur son plan de travail.
fn preview(app: &App) -> Element<'_, Message> {
    let bar = document::workbench_bar(
        "Aperçu du document",
        row![typo::caption("A4"), zoom_controls(app),]
            .spacing(space::MD)
            .align_y(Alignment::Center),
    );

    let page: Element<'_, Message> = match &app.cv_generation {
        Some(generation) => page_content(app, generation, app.cv_version_name.clone()),
        None => placeholder(app),
    };

    container(
        column![
            container(bar).width(Length::Fill),
            surface::divider(),
            document::workspace(document::page(app.document_width, page)),
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|theme: &Theme| {
        let palette = tokens(theme);
        container::Style {
            background: Some(Background::Color(palette.panel)),
            border: Border {
                color: palette.border,
                width: stroke::HAIRLINE,
                ..Border::default()
            },
            ..container::Style::default()
        }
    })
    .into()
}

fn zoom_controls(app: &App) -> Element<'_, Message> {
    let mut segments = Vec::new();
    for width in document::zoom_widths() {
        segments.push(
            controls::segment(
                format!("{} %", document::zoom_percent(width)),
                (app.document_width - width).abs() < f32::EPSILON,
            )
            .on_press(Message::DocumentWidthChanged(width)),
        );
    }
    controls::segmented(segments)
}

/// Contenu de la page A4 pour une génération donnée.
pub fn page_content<'a>(
    app: &'a App,
    generation: &'a CvGeneration,
    _title: String,
) -> Element<'a, Message> {
    let profile = &app.data.profile.personal;
    let mut page = column![
        document::heading(format!("{} {}", profile.first_name, profile.last_name).trim()),
        document::subheading(crate::ui::format::or_else(
            profile.headline.as_deref(),
            "Titre professionnel"
        )),
        document::body_muted(contact_line(app)),
        iced::widget::Space::with_height(space::LG),
        document::rubric("Profil"),
        document::body(generation.cv.summary.clone()),
    ]
    .spacing(space::SM)
    .width(Length::Fill);

    if !generation.cv.skills.is_empty() {
        page = page
            .push(iced::widget::Space::with_height(space::SM))
            .push(document::rubric("Compétences"))
            .push(document::body(generation.cv.skills.join(" · ")));
    }

    if !generation.cv.experiences.is_empty() {
        page = page
            .push(iced::widget::Space::with_height(space::SM))
            .push(document::rubric("Expériences"));
        for experience in &generation.cv.experiences {
            page = page.push(document::entry(
                format!("{} — {}", experience.title, experience.company),
                String::new(),
                experience.description.clone(),
            ));
        }
    }

    if !generation.cv.education.is_empty() {
        page = page
            .push(iced::widget::Space::with_height(space::SM))
            .push(document::rubric("Formation"));
        for education in &generation.cv.education {
            page = page.push(document::entry(
                education.degree.clone(),
                String::new(),
                education.school.clone(),
            ));
        }
    }

    page.into()
}

fn placeholder(app: &App) -> Element<'_, Message> {
    let profile = &app.data.profile.personal;
    let name = format!("{} {}", profile.first_name, profile.last_name)
        .trim()
        .to_owned();
    column![
        document::heading(if name.is_empty() {
            "Votre nom".to_owned()
        } else {
            name
        }),
        document::subheading(crate::ui::format::or_else(
            profile.headline.as_deref(),
            "Titre professionnel"
        )),
        document::body_muted(contact_line(app)),
        iced::widget::Space::with_height(space::MAX),
        document::rubric("Profil"),
        document::body(crate::ui::format::or_else(
            profile.summary.as_deref(),
            "Analysez une offre pour générer un CV adapté à sa fiche de poste.",
        )),
        iced::widget::Space::with_height(space::MD),
        document::body_muted(format!(
            "{} · {}",
            crate::ui::format::plural(
                app.data.profile.experiences.len(),
                "expérience",
                "expériences"
            ),
            crate::ui::format::plural(app.data.profile.skills.len(), "compétence", "compétences"),
        )),
    ]
    .spacing(space::SM)
    .width(Length::Fill)
    .into()
}

fn contact_line(app: &App) -> String {
    let personal = &app.data.profile.personal;
    [
        Some(personal.email.as_str()).filter(|value| !value.is_empty()),
        personal.phone.as_deref(),
        personal.city.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter(|value| !value.trim().is_empty())
    .collect::<Vec<_>>()
    .join(" · ")
}

/// Compétences de l'offre présentes dans le profil, prêtes à afficher.
///
/// L'analyse fournit déjà les listes `matched`/`missing` : on ne recalcule pas
/// le score, on nettoie seulement les doublons (casse ignorée, ordre préservé).
fn present_skills(score: &MatchScore) -> Vec<String> {
    clean_skills(&score.matched)
}

/// Compétences de l'offre absentes du profil, prêtes à afficher.
fn missing_skills(score: &MatchScore) -> Vec<String> {
    clean_skills(&score.missing)
}

/// Déduplication insensible à la casse : premier exemplaire conservé, ordre préservé.
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

#[cfg(test)]
mod tests {
    use super::{
        detected_company, missing_skills, panel_footer_state, present_skills, PanelFooterState,
    };
    use crate::modules::entreprises::model::Entreprise;
    use crate::modules::ia::cv_model::MatchScore;

    fn entreprise(nom: &str) -> Entreprise {
        Entreprise {
            id: uuid::Uuid::nil(),
            nom: nom.to_owned(),
            secteur: None,
            type_: None,
            site_web: None,
            ville: None,
            adresse: None,
            notes: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn score(matched: &[&str], missing: &[&str]) -> MatchScore {
        MatchScore {
            total: 0,
            skills: 0,
            experience: 0,
            ats: 0,
            matched: matched.iter().map(|skill| (*skill).to_owned()).collect(),
            missing: missing.iter().map(|skill| (*skill).to_owned()).collect(),
        }
    }

    fn owned(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn les_doublons_sont_supprimes() {
        let analysis = score(&["Rust", "Rust", "Go"], &["SQL", "SQL"]);
        assert_eq!(present_skills(&analysis), owned(&["Rust", "Go"]));
        assert_eq!(missing_skills(&analysis), owned(&["SQL"]));
    }

    #[test]
    fn sans_analyse_le_pied_est_vide() {
        assert_eq!(
            panel_footer_state(false, false, false),
            PanelFooterState::None
        );
        assert_eq!(
            panel_footer_state(false, true, false),
            PanelFooterState::None
        );
        assert_eq!(
            panel_footer_state(false, false, true),
            PanelFooterState::None
        );
    }

    #[test]
    fn analyse_seule_propose_la_generation() {
        assert_eq!(
            panel_footer_state(true, false, false),
            PanelFooterState::ProposeGeneration
        );
    }

    #[test]
    fn generation_en_cours_affiche_la_progression() {
        assert_eq!(
            panel_footer_state(true, true, false),
            PanelFooterState::Generating
        );
    }

    #[test]
    fn generation_terminee_laisse_place_aux_suggestions() {
        assert_eq!(
            panel_footer_state(true, false, true),
            PanelFooterState::None
        );
    }

    #[test]
    fn regeneration_conserve_les_suggestions_affichees() {
        assert_eq!(panel_footer_state(true, true, true), PanelFooterState::None);
    }

    #[test]
    fn la_casse_est_ignoree() {
        let analysis = score(&["Rust", "rust"], &["Go", "GO", "go"]);
        assert_eq!(present_skills(&analysis), owned(&["Rust"]));
        assert_eq!(missing_skills(&analysis), owned(&["Go"]));
    }

    #[test]
    fn l_ordre_d_apparition_est_preserve() {
        let analysis = score(&["C", "Rust", "C++"], &["Kafka", "SQL", "Kafka"]);
        assert_eq!(present_skills(&analysis), owned(&["C", "Rust", "C++"]));
        assert_eq!(missing_skills(&analysis), owned(&["Kafka", "SQL"]));
    }

    #[test]
    fn une_liste_vide_reste_vide() {
        let analysis = score(&[], &[]);
        assert!(present_skills(&analysis).is_empty());
        assert!(missing_skills(&analysis).is_empty());
    }

    #[test]
    fn detecte_une_entreprise_citee_dans_l_offre() {
        let companies = vec![entreprise("Acme Corp"), entreprise("Globex")];
        assert_eq!(
            detected_company("Rejoignez Acme Corp à Paris.", &companies),
            Some("Acme Corp".to_owned())
        );
    }

    #[test]
    fn la_detection_ignore_la_casse() {
        let companies = vec![entreprise("Acme Corp")];
        assert_eq!(
            detected_company("Stage développeur chez ACME CORP.", &companies),
            Some("Acme Corp".to_owned())
        );
    }

    #[test]
    fn un_texte_vide_ne_detecte_rien() {
        let companies = vec![entreprise("Acme Corp")];
        assert_eq!(detected_company("", &companies), None);
        assert_eq!(detected_company("   ", &companies), None);
    }

    #[test]
    fn aucune_entreprise_connue_ne_detecte_rien() {
        let companies = vec![entreprise("Acme Corp")];
        assert_eq!(
            detected_company("Poste au sein d'une startup inconnue.", &companies),
            None
        );
    }

    #[test]
    fn l_entreprise_la_plus_tot_est_retournee() {
        let companies = vec![entreprise("Acme Corp"), entreprise("Corp Global")];
        assert_eq!(
            detected_company("Chez Corp Global comme chez Acme Corp.", &companies),
            Some("Corp Global".to_owned())
        );
    }

    #[test]
    fn la_detection_ne_reconnait_pas_un_nom_court_dans_un_mot() {
        let companies = vec![entreprise("Dev"), entreprise("Globex")];
        assert_eq!(
            detected_company("Développeur backend chez Globex.", &companies),
            Some("Globex".to_owned())
        );
    }

    #[test]
    fn le_nom_compose_prime_sur_un_sous_nom() {
        let companies = vec![entreprise("Corp"), entreprise("Corp Global")];
        assert_eq!(
            detected_company("Rejoignez Corp Global.", &companies),
            Some("Corp Global".to_owned())
        );
    }
}
