//! Point d'entrée natif de Candilog Desktop.

use candilog::app::{self, App};
use iced::{window, Font, Size};

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("candilog=info")),
        )
        .init();

    let window_size = match std::env::var("CANDILOG_CAPTURE_SIZE").as_deref() {
        Ok("small") => Size::new(1100.0, 700.0),
        Ok("large") => Size::new(1800.0, 1100.0),
        _ => Size::new(1440.0, 900.0),
    };

    iced::application("Candilog", app::update, app::view)
        .font(include_bytes!("../assets/fonts/Cantarell-VF.otf").as_slice())
        .default_font(Font::with_name("Cantarell"))
        .antialiasing(true)
        .theme(app::theme)
        .subscription(app::subscription)
        .window(window::Settings {
            size: window_size,
            min_size: Some(Size::new(1040.0, 660.0)),
            ..window::Settings::default()
        })
        .run_with(App::new)
}
