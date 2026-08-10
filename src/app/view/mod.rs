//! Coquille de l'application : fond ambiant, barre de titre, barre latérale et panneau.
//!
//! Aucun écran métier n'est rendu ici : chaque module expose sa propre vue.

mod dialogs;

use super::{App, Message};
use crate::navigation::Route;
use crate::ui::components::ambient;
use crate::ui::components::runtime_status::{app_version, runtime_status, Health};
use crate::ui::components::sidebar::sidebar;
use crate::ui::components::skeleton::PageSkeleton;
use crate::ui::components::titlebar::titlebar;
use crate::ui::components::{notification, overlay, state};
use crate::ui::theme::styles;
use iced::widget::{column, container, row, stack};
use iced::{Element, Length};

/// Rend l'application complète.
pub fn view(app: &App) -> Element<'_, Message> {
    if let Some(error) = &app.fatal_error {
        return state::fatal(error, "Réessayer", Message::Reload);
    }

    let page: Element<'_, Message> = if app.initialized {
        screen(app)
    } else {
        PageSkeleton::Dashboard.render()
    };

    let main = container(page)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(styles::glass_panel);

    let shell_body = column![
        titlebar(
            runtime_status(
                provider_label(app),
                model_label(app),
                if app.ai_is_running {
                    Health::Checking
                } else {
                    Health::Ok
                },
            ),
            app.is_dark,
            Message::ToggleTheme,
        ),
        row![sidebar(app.route, Message::Navigate, app_version()), main,]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(8.0)
            .padding(iced::Padding::default().right(8.0).bottom(8.0)),
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let mut layers: Vec<Element<'_, Message>> =
        vec![stack(vec![ambient::ambient(), shell_body.into()]).into()];
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
    stack(layers)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
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

fn provider_label(app: &App) -> &str {
    let provider = &app.data.settings.llm.provider;
    match provider {
        crate::shared::llm::ProviderKind::Ollama => "ollama",
        crate::shared::llm::ProviderKind::Claude => "claude",
        crate::shared::llm::ProviderKind::OpenAI => "openai",
        crate::shared::llm::ProviderKind::Gemini => "gemini",
        crate::shared::llm::ProviderKind::Mistral => "mistral",
        crate::shared::llm::ProviderKind::Nvidia => "nvidia",
        crate::shared::llm::ProviderKind::Custom(_) => "custom",
    }
}

fn model_label(app: &App) -> &str {
    app.data.settings.llm.model.as_str()
}

/// Ferme le dialogue courant depuis n'importe quelle couche superposée.
pub(super) const DISMISS: Message = Message::CloseDialog;

/// Rend l'inspection d'une candidature : drawer superposé (460 px).
pub(super) fn inspector_layer(app: &App, id: uuid::Uuid) -> Element<'_, Message> {
    let content = crate::modules::candidatures::views::inspector::view(app, id);
    overlay::drawer(content, DISMISS)
}
