//! Point d'entrée natif de Candilog Desktop.

use candilog::app::{self, App};
use iced::{window, Font, Size};

fn main() -> iced::Result {
    // `_garde` doit vivre jusqu'à la fin de `main` : c'est elle qui vide le tampon du journal
    // fichier à l'arrêt.
    let _garde = candilog::core::logging::initialiser();

    let window_size = if cfg!(feature = "capture") {
        match std::env::var("CANDILOG_CAPTURE_SIZE").as_deref() {
            Ok("small") => Size::new(1100.0, 700.0),
            Ok("large") => Size::new(1800.0, 1100.0),
            _ => Size::new(1440.0, 900.0),
        }
    } else {
        Size::new(1440.0, 900.0)
    };

    iced::application("Candilog", app::update, app::view)
        .font(include_bytes!("../assets/fonts/Geist[wght].ttf").as_slice())
        .font(include_bytes!("../assets/fonts/GeistMono[wght].ttf").as_slice())
        .default_font(Font::with_name("Geist"))
        .antialiasing(true)
        .theme(app::theme)
        .subscription(app::subscription)
        .window(window::Settings {
            size: window_size,
            min_size: Some(Size::new(800.0, 600.0)),
            // Sans icône, la fenêtre n'en a aucune dans la barre des tâches et le sélecteur
            // de fenêtres, alors que le jeu d'icônes de l'application existe au dépôt.
            icon: candilog::core::logging::icone_application(),
            ..window::Settings::default()
        })
        .run_with(App::new)
}
