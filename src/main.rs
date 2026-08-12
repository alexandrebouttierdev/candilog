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

    let mut window = window::Settings {
        size: window_size,
        min_size: Some(Size::new(800.0, 600.0)),
        // Icône de fenêtre embarquée pour X11/Windows et les sélecteurs qui la lisent.
        icon: candilog::core::logging::icone_application(),
        ..window::Settings::default()
    };
    #[cfg(target_os = "linux")]
    {
        // Sous Wayland, le shell retrouve l'icône via l'identifiant du fichier
        // `candilog.desktop` plutôt que via l'icône attachée à la fenêtre.
        window.platform_specific.application_id = "candilog".to_owned();
    }

    iced::application("Candilog", app::update, app::view)
        .font(include_bytes!("../assets/fonts/Geist[wght].ttf").as_slice())
        .font(include_bytes!("../assets/fonts/GeistMono[wght].ttf").as_slice())
        .default_font(Font::with_name("Geist"))
        .antialiasing(true)
        .theme(app::theme)
        .subscription(app::subscription)
        .window(window)
        .run_with(App::new)
}
