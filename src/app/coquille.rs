//! Abonnements globaux, raccourcis clavier et thème : la coquille de l'application,
//! distincte du traitement des messages métier.

use super::{App, Message};
use iced::{keyboard, Subscription, Theme};
use std::time::Duration;

/// Abonnements desktop : chronomètre et fermeture clavier des surfaces temporaires.
pub fn subscription(app: &App) -> Subscription<Message> {
    let mut abonnements = vec![
        keyboard::on_key_press(shortcut),
        iced::window::resize_events().map(|(_id, size)| Message::WindowResized(size)),
    ];
    // Le chronomètre ne sert qu'aux opérations IA. Inconditionnel, il délivrait un message par
    // seconde en permanence — et dans le modèle Iced tout message provoque un cycle
    // `update` + `view`, donc la reconstruction complète de l'arbre de widgets de l'écran
    // courant, avec le filtrage des candidatures et toutes les allocations de libellés que
    // cela suppose. Sur une application censée rester ouverte en arrière-plan, c'était une
    // consommation CPU permanente et entièrement évitable.
    if app.ai_is_running {
        abonnements.push(iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick));
    }
    // Le compte à rebours du toast n'est abonné que lorsqu'un toast est affiché : sans
    // notification, aucun message périodique ne tourne en arrière-plan (même principe que
    // le chronomètre IA ci-dessus). La précision réelle vient de `Instant`, la période de
    // 500 ms ne fait que cadencer la vérification.
    if app.notification.is_some() {
        abonnements.push(
            iced::time::every(Duration::from_millis(500)).map(|_| Message::NotificationCountdown),
        );
    }
    Subscription::batch(abonnements)
}

/// Traduit une combinaison clavier en message applicatif.
fn shortcut(key: keyboard::Key, modifiers: keyboard::Modifiers) -> Option<Message> {
    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) {
        return Some(Message::DismissTopLayer);
    }
    if !modifiers.command() {
        return None;
    }
    let keyboard::Key::Character(character) = &key else {
        return None;
    };
    match character.as_str() {
        "n" => Some(Message::OpenDialog(super::state::Dialog::Candidature)),
        "f" => Some(Message::FocusSearch),
        "r" => Some(Message::Reload),
        digit => digit
            .chars()
            .next()
            .filter(char::is_ascii_digit)
            .and_then(crate::navigation::Route::from_shortcut)
            .map(Message::Navigate),
    }
}

/// Thème Iced actif.
pub fn theme(app: &App) -> Theme {
    if app.is_dark {
        crate::ui::theme::dark()
    } else {
        crate::ui::theme::light()
    }
}
