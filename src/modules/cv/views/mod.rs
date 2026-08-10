//! Écran Mes CV : bibliothèque en grille à gauche, aperçu du document à droite.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::cv::components::{self, version_card};
use crate::navigation::Route;
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::Icon;
use crate::ui::components::stat_card;
use crate::ui::components::{document, field, layout, meter, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Rend l'écran de la bibliothèque de CV.
pub fn view(app: &App) -> Element<'_, Message> {
    let actions = controls::primary("Générateur de CV", Some(Icon::Sparkles))
        .on_press(Message::Navigate(Route::CvGenerator))
        .into();

    let metrics = row![
        stat_card::metric_icon_tinted(
            "Versions sauvegardées",
            app.data.cv_versions.len().to_string(),
            Tone::Accent,
            Icon::Document,
        ),
        stat_card::metric_icon_tinted(
            "Dernière mise à jour",
            components::latest_version_date(&app.data.cv_versions),
            Tone::Info,
            Icon::Clock,
        ),
    ]
    .spacing(space::MD);

    layout::screen(
        header::page_header(
            Icon::Document,
            "Mes CV",
            "Votre bibliothèque de versions",
            actions,
        ),
        layout::workspace(
            column![metrics, layout::split(grid(app), preview(app))]
                .spacing(space::LG)
                .height(Length::Fill),
        ),
    )
}

/// Grille des versions filtrées par la recherche.
fn grid(app: &App) -> Element<'_, Message> {
    let needle = app.search.trim().to_lowercase();
    let versions: Vec<_> = app
        .data
        .cv_versions
        .iter()
        .filter(|version| components::matches(version, &needle))
        .collect();

    let body: Element<'_, Message> = if versions.is_empty() {
        state::empty(
            "Aucune version",
            "Sauvegardez une version depuis le générateur ou modifiez votre recherche.",
        )
    } else {
        let mut cards = row![].spacing(space::MD);
        for version in versions {
            cards = cards.push(
                container(version_card(
                    version,
                    Message::LoadCvVersion(version.id),
                    Message::LoadCvVersion(version.id),
                    Message::OpenDialog(Dialog::DeleteCv(version.id)),
                ))
                .width(Length::FillPortion(1))
                .max_width(240.0),
            );
        }
        surface::scroll(cards.wrap()).height(Length::Fill).into()
    };

    container(
        column![
            container(field::search(
                "Rechercher une version…",
                &app.search,
                Message::SearchChanged,
                Length::Fill,
            ))
            .padding(space::LG),
            surface::divider(),
            body,
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(styles::glass_card)
    .into()
}

/// Aperçu du document de la version sélectionnée.
fn preview(app: &App) -> Element<'_, Message> {
    let Some(version) = app.focused_cv() else {
        return state::no_selection("Sélectionnez une version pour afficher son aperçu.");
    };

    let bar = document::workbench_bar(
        "Aperçu du document",
        row![
            typo::caption(format::compact_datetime(&version.created_at)),
            zoom_controls(app),
            controls::ghost("Charger", Some(Icon::Open))
                .on_press(Message::LoadCvVersion(version.id)),
            controls::ghost("Exporter", Some(Icon::Download)).on_press_maybe(
                app.cv_generation
                    .is_some()
                    .then_some(Message::ExportGeneratedCvPdf),
            ),
            controls::icon_danger(
                Icon::Trash,
                "Supprimer",
                Message::OpenDialog(Dialog::DeleteCv(version.id)),
            ),
        ]
        .spacing(space::SM)
        .align_y(Alignment::Center),
    );

    let page = match &app.cv_generation {
        Some(generation) => super::super::ia::views::cv_generator::page_content(
            app,
            generation,
            version.name.clone(),
        ),
        None => empty_page(version.name.clone()),
    };

    column![
        container(bar).width(Length::Fill),
        surface::divider(),
        document::workspace(document::page(app.document_width, page)),
    ]
    .height(Length::Fill)
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

fn empty_page<'a>(name: String) -> Element<'a, Message> {
    column![
        document::heading(name),
        document::subheading("Version enregistrée"),
        iced::widget::Space::with_height(space::MAX),
        document::body("Chargez cette version pour afficher son contenu détaillé dans l'aperçu.",),
        iced::widget::Space::with_height(space::LG),
        meter::ats(0),
    ]
    .spacing(space::MD)
    .width(Length::Fill)
    .into()
}
