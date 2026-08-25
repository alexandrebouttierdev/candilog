//! Écran « Mes lettres de motivation ».

use crate::app::state::Dialog;
use crate::app::{App, Message};
use crate::modules::lettres::components;
use crate::navigation::Route;
use crate::ui::components::button as controls;
use crate::ui::components::header;
use crate::ui::components::icon::Icon;
use crate::ui::components::{badge, layout, list, state, surface, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, text};
use iced::{Alignment, Element, Length};

/// Bibliothèque persistante, avec liste à gauche et lecture à droite.
pub fn view(app: &App) -> Element<'_, Message> {
    let compact = app.layout().width < 1_280.0;
    let actions: Element<'_, Message> = if compact {
        controls::icon_primary(
            Icon::Sparkles,
            "Rédiger une lettre",
            Message::Navigate(Route::LettreMotivation),
        )
    } else {
        controls::primary("Rédiger une lettre", Some(Icon::Sparkles))
            .on_press(Message::Navigate(Route::LettreMotivation))
            .into()
    };
    layout::screen(
        header::page_header(
            Icon::Mail,
            if compact {
                "Mes lettres"
            } else {
                "Mes lettres de motivation"
            },
            "Votre bibliothèque de lettres",
            actions,
        ),
        layout::workspace(layout::split_portions(5, library(app), 8, preview(app))),
    )
}

fn library(app: &App) -> Element<'_, Message> {
    let needle = app.search.trim().to_lowercase();
    let letters: Vec<_> = app
        .data
        .letters
        .iter()
        .filter(|letter| components::matches(letter, &needle))
        .collect();

    let body: Element<'_, Message> = if letters.is_empty() {
        state::empty(
            "Aucune lettre enregistrée",
            "Rédigez une lettre puis enregistrez-la dans votre bibliothèque.",
        )
    } else {
        let mut rows = column![].spacing(space::SM);
        for letter in letters {
            let metadata = format!(
                "{} · {}",
                format::or_else(letter.company.as_deref(), "Entreprise non renseignée"),
                format::compact_datetime(&letter.created_at),
            );
            rows = rows.push(list::row_item(
                crate::ui::components::icon::muted(Icon::Mail),
                letter.name.clone(),
                metadata,
                badge::badge(
                    format::or_else(letter.job_title.as_deref(), "Candidature"),
                    Tone::Neutral,
                ),
                app.selected_letter == Some(letter.id),
                Message::SelectLetter(Some(letter.id)),
            ));
        }
        surface::scroll(container(rows).padding(space::MD))
            .height(Length::Fill)
            .into()
    };

    let toolbar = column![row![
        typo::label("Bibliothèque"),
        layout::spacer(),
        typo::caption(format::plural(app.data.letters.len(), "lettre", "lettres")),
    ]
    .align_y(Alignment::Center),]
    .spacing(space::MD);

    container(
        column![
            container(toolbar).padding(space::XL),
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

fn preview(app: &App) -> Element<'_, Message> {
    let Some(letter) = app.focused_letter() else {
        return state::no_selection("Sélectionnez une lettre pour la relire.");
    };
    let header = row![
        column![
            typo::section(letter.name.clone()),
            typo::caption(format!(
                "{} · {}",
                format::or_else(letter.company.as_deref(), "Entreprise"),
                format::compact_datetime(&letter.created_at),
            )),
        ]
        .spacing(space::XXS),
        layout::spacer(),
        controls::secondary("Modifier", Some(Icon::Edit)).on_press(Message::LoadLetter(letter.id)),
        controls::icon_danger(
            Icon::Trash,
            "Supprimer",
            Message::OpenDialog(Dialog::DeleteLetter(letter.id)),
        ),
    ]
    .spacing(space::SM)
    .align_y(Alignment::Center);

    let page = container(
        column![
            typo::meta_toned("LETTRE DE MOTIVATION", Tone::Accent),
            typo::title(format::or_else(letter.job_title.as_deref(), "Candidature")),
            typo::caption(format::or_else(letter.company.as_deref(), "Entreprise")),
            surface::divider(),
            text(letter.content.clone())
                .size(13.5)
                .line_height(iced::widget::text::LineHeight::Absolute(24.0.into())),
        ]
        .spacing(space::LG),
    )
    .max_width(720.0)
    .padding(40.0)
    .style(styles::document_paper);

    container(
        column![
            container(header).padding([space::MD, space::XL]),
            surface::divider(),
            surface::scroll(container(page).center_x(Length::Fill).padding(space::MAX))
                .height(Length::Fill),
        ]
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(styles::glass_card)
    .into()
}
