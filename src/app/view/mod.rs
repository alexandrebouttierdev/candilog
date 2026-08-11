//! Coquille de l'application : rail, barre d'état, navigation contextuelle et contenu.
//!
//! Aucun écran métier n'est rendu ici : chaque module expose sa propre vue.

mod date_picker;
mod dialogs;

use super::{App, Message};
use crate::navigation::Route;
use crate::ui::components::icon::Icon;
use crate::ui::components::runtime_status::{app_version, runtime_status, Health};
use crate::ui::components::sidebar::sidebar;
use crate::ui::components::skeleton::PageSkeleton;
use crate::ui::components::titlebar::titlebar;
use crate::ui::components::{notification, overlay, state};
use crate::ui::theme::styles;
use iced::widget::{column, container, mouse_area, row, stack};
use iced::{Element, Length};

/// Rend l'application complète.
pub fn view(app: &App) -> Element<'_, Message> {
    if let Some(error) = &app.fatal_error {
        return state::fatal(
            error,
            vec![
                ("Réessayer", Icon::Refresh, Message::RetryBootstrap),
                (
                    "Restaurer un backup",
                    Icon::Download,
                    Message::SelectRecoveryBackup,
                ),
                (
                    "Redémarrer sur une base neuve",
                    Icon::Trash,
                    Message::QuarantineDatabase,
                ),
            ],
        );
    }

    let page: Element<'_, Message> = if app.initialized {
        screen(app)
    } else {
        // Le squelette annonce la structure à venir : il suit donc la route active. Servir
        // celui du tableau de bord sur toutes les routes amplifie le saut au chargement au
        // lieu de l'atténuer.
        skeleton_for(app.route).render()
    };

    let main = container(page)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(styles::canvas);

    let runtime = runtime_status(
        provider_label(app),
        model_label(app),
        if app.ai_is_running {
            Health::Checking
        } else {
            app.provider_health
        },
    );
    let content = column![titlebar(runtime), main,]
        .width(Length::Fill)
        .height(Length::Fill);

    let shell_body = row![
        sidebar(
            app.route,
            Message::Navigate,
            app.is_dark,
            Message::ToggleTheme,
            app_version(),
            Message::Navigate(Route::APropos),
        ),
        content,
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    let mut layers: Vec<Element<'_, Message>> = vec![shell_body.into()];
    if let Some(dialog) = app.dialog {
        layers.push(dialogs::layer(app, dialog));
    }
    if app.date_picker.is_some() {
        layers.push(date_picker::layer(app));
    }
    if let Some(notice) = &app.notification {
        layers.push(notification::toast(
            notice.message.clone(),
            notice.kind,
            Message::ClearNotification,
        ));
    }
    mouse_area(stack(layers).width(Length::Fill).height(Length::Fill))
        .on_release(Message::CandidateDragCancelled)
        .into()
}

/// Squelette annonçant la structure de la route en cours de chargement.
///
/// Calqué sur le `match` de [`screen`] : c'est la même correspondance route → mise en page,
/// à ceci près qu'elle porte sur la forme d'attente.
const fn skeleton_for(route: Route) -> PageSkeleton {
    match route {
        Route::Dashboard => PageSkeleton::Dashboard,
        Route::Candidatures => PageSkeleton::Board,
        Route::Calendrier => PageSkeleton::Calendar,
        Route::Statistiques => PageSkeleton::Default,
        Route::Entreprises | Route::Reseau => PageSkeleton::List,
        Route::Cv => PageSkeleton::Cards,
        Route::CvGenerator | Route::LettreMotivation | Route::CvImport => PageSkeleton::Default,
        Route::Profil
        | Route::Parametres
        | Route::Sauvegardes
        | Route::MisesAJour
        | Route::APropos => PageSkeleton::Form,
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
        Route::Sauvegardes => crate::modules::settings::views::backup_view(app),
        Route::MisesAJour => crate::modules::settings::views::updates_view(app),
        Route::APropos => crate::modules::settings::views::about_view(app),
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
