//! Shell de l'application : navigation, écran actif, dialogues et notifications.
//!
//! Ce module ne rend aucun écran métier : chaque module expose sa propre vue.

mod dialogs;

use super::{App, Message};
use crate::navigation::Route;
use crate::ui::components::icon::Icon;
use crate::ui::components::pane::pane;
use crate::ui::components::rail::{rail, Load};
use crate::ui::components::{icon, notification, overlay, state, surface, toolbar, typo};
use crate::ui::format;
use crate::ui::theme::metrics::space;
use crate::ui::theme::styles;
use crate::ui::theme::Tone;
use iced::widget::{column, container, row, stack};
use iced::{Alignment, Element, Length};

/// Rend l'application complète.
pub fn view(app: &App) -> Element<'_, Message> {
    if let Some(error) = &app.fatal_error {
        return state::fatal(error, "Réessayer", Message::Reload);
    }

    let workspace = column![screen(app), status_bar(app)]
        .width(Length::Fill)
        .height(Length::Fill);

    let section = app.route.section();
    let mut bands = row![rail(
        section,
        load(app),
        app.layout(),
        |section: crate::navigation::Section| Message::Navigate(section.default_route()),
        Message::ToggleTheme,
        app.is_dark,
    )]
    .height(Length::Fill);

    if let Some(volet) = pane(section, app.route, load(app), Message::Navigate) {
        bands = bands.push(volet);
    }

    let shell: Element<'_, Message> = bands
        .push(
            container(workspace)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(styles::canvas),
        )
        .into();

    let mut layers = vec![shell];
    if let Some(dialog) = app.dialog {
        layers.push(dialogs::layer(app, dialog));
    }
    if let Some(message) = &app.notification {
        layers.push(notification::toast(
            message.clone(),
            notification::Kind::infer(message),
            Message::ClearNotification,
        ));
    }

    if layers.len() == 1 {
        layers.pop().unwrap_or_else(|| Element::from(row![]))
    } else {
        stack(layers)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn screen(app: &App) -> Element<'_, Message> {
    match app.route {
        Route::Dashboard => crate::modules::metriques::views::dashboard_view(app),
        Route::Candidatures => crate::modules::candidatures::views::view(app),
        Route::Calendrier => crate::modules::entretiens::views::view(app),
        Route::Statistiques => crate::modules::metriques::views::statistics_view(app),
        Route::Entreprises => crate::modules::entreprises::views::view(app),
        Route::Reseau => crate::modules::contacts::views::view(app),
        Route::Cv => crate::modules::cv::views::view(app),
        Route::CvGenerator => crate::modules::ia::views::cv_generator::view(app),
        Route::LettreMotivation => crate::modules::ia::views::cover_letter::view(app),
        Route::CvImport => crate::modules::ia::views::cv_import::view(app),
        Route::Profil => crate::modules::profil::views::view(app),
        Route::Parametres => crate::modules::settings::views::view(app),
    }
}

fn load(app: &App) -> Load {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    Load {
        candidatures: app
            .data
            .candidatures
            .iter()
            .filter(|item| {
                item.statut == crate::modules::candidatures::model::StatutCandidature::EnAttente
            })
            .count(),
        agenda: app.upcoming_interviews(&today),
        relances: app.due_reminders(&today),
    }
}

fn status_bar(app: &App) -> Element<'_, Message> {
    let database = app.paths.as_ref().map_or("initialisation", |paths| {
        paths
            .database
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("candilog.sqlite")
    });

    let left = row![
        icon::toned(
            Icon::Check,
            if app.ai_is_running {
                Tone::Warning
            } else {
                Tone::Success
            },
        ),
        typo::micro(if app.ai_is_running {
            "Opération IA en cours"
        } else {
            "Prêt"
        }),
        surface::split_rule(),
        typo::micro(format!("Base : {database}")),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center);

    let right = row![
        typo::micro(format::plural(
            app.data.candidatures.len(),
            "candidature",
            "candidatures"
        )),
        surface::split_rule(),
        typo::micro(format::plural(
            app.data.entreprises.len(),
            "entreprise",
            "entreprises"
        )),
        surface::split_rule(),
        typo::micro(concat!("Candilog ", env!("CARGO_PKG_VERSION"))),
    ]
    .spacing(space::MD)
    .align_y(Alignment::Center);

    toolbar::status_bar(left, right)
}

/// Ferme le dialogue courant depuis n'importe quelle couche superposée.
pub(super) const DISMISS: Message = Message::CloseDialog;

/// Rend le drawer d'inspection d'une candidature.
pub(super) fn inspector_layer(app: &App, id: uuid::Uuid) -> Element<'_, Message> {
    overlay::drawer(
        crate::modules::candidatures::views::inspector::view(app, id),
        DISMISS,
    )
}
