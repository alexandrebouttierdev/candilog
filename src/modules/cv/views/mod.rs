//! Écran Mes CV : bibliothèque à gauche, aperçu du document à droite.

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::cv::components::version_row;
use crate::navigation::Route;
use crate::ui::components::button as controls;
use crate::ui::components::icon::Icon;
use crate::ui::components::{badge, document, layout, meter, state, surface, toolbar, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use iced::widget::{column, container, row};
use iced::{Alignment, Element, Length};

/// Rend l'écran de la bibliothèque de CV.
pub fn view(app: &App) -> Element<'_, Message> {
    let leading = toolbar::group([
        badge::count(app.data.cv_versions.len()),
        toolbar::separator(),
        typo::meta(crate::modules::cv::components::library_summary(
            app.data.cv_versions.len(),
        ))
        .into(),
    ]);
    let export = controls::ghost("Exporter en PDF", Some(Icon::Download));
    let trailing = toolbar::group([
        if app.cv_generation.is_some() {
            export.on_press(Message::ExportGeneratedCvPdf).into()
        } else {
            export.into()
        },
        controls::primary("Ouvrir le générateur", Some(Icon::Sparkles))
            .on_press(Message::Navigate(Route::CvGenerator))
            .into(),
    ]);

    layout::screen(
        toolbar::toolbar("Mes CV", leading, trailing),
        layout::split(library(app), preview(app)),
    )
}

fn library(app: &App) -> Element<'_, Message> {
    let body: Element<'_, Message> = if app.data.cv_versions.is_empty() {
        state::empty(
            "Aucune version enregistrée",
            "Les CV sauvegardés depuis le générateur apparaîtront ici.",
        )
    } else {
        let mut rows = column![];
        for version in &app.data.cv_versions {
            rows = rows.push(version_row(
                version,
                app.selected_cv == Some(version.id),
                Message::SelectCvVersion(Some(version.id)),
            ));
        }
        surface::scroll(rows).height(Length::Fill).into()
    };
    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(crate::ui::theme::styles::panel_flat)
        .into()
}

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
