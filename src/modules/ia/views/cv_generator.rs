//! CV Generator : workflow, jauge de score et atelier d'analyse à gauche,
//! document A4 posé à droite.

use crate::app::{App, Message};
use crate::navigation::Route;
use crate::ui::components::header;
use crate::ui::components::icon::Icon;
use crate::ui::components::layout;
use crate::ui::theme::metrics::space;
use iced::widget::column;
use iced::{Element, Length};

pub mod preview;
pub mod workbench;
pub mod workflow;

pub use preview::page_content;
use preview::preview;
use workbench::workbench;
use workflow::{actions, overview};

#[cfg(test)]
#[path = "tests/cv_generator/mod.rs"]
mod tests;

/// Rend l'écran du générateur de CV.
pub fn view(app: &App) -> Element<'_, Message> {
    layout::screen(
        header::route_header(
            Icon::Sparkles,
            "Générer un CV",
            Route::CvGenerator,
            Message::Navigate,
            actions(app),
        ),
        layout::workspace(
            column![
                overview(app),
                layout::split_portions(7, workbench(app), 8, preview(app)),
            ]
            .spacing(space::LG)
            .height(Length::Fill),
        ),
    )
}
