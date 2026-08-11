//! Harnais de capture visuelle, réservé à la revue de design.
//!
//! Isolé derrière la caractéristique Cargo `capture` : ces branches écrivent un fichier au
//! chemin indiqué par l'environnement, avec les droits de l'utilisateur, et n'ont rien à faire
//! dans le binaire distribué.

pub(super) fn save_review_screenshot(screenshot: &iced::window::Screenshot) -> Result<(), String> {
    // Sans la caractéristique `capture`, aucune écriture de fichier n'est possible par ce
    // chemin : le binaire distribué ne doit pas offrir à son environnement le moyen d'écrire
    // où bon lui semble sous l'identité de l'utilisateur.
    if !crate::app::state::capture_demandee() {
        return Err("Harnais de capture visuelle absent de cette version.".into());
    }
    let path = std::env::var_os("CANDILOG_CAPTURE_PATH")
        .ok_or_else(|| "Chemin de capture visuelle absent.".to_string())?;
    let pixel_count = u64::from(screenshot.size.width) * u64::from(screenshot.size.height);
    let expected = usize::try_from(pixel_count.saturating_mul(4))
        .map_err(|_| "Capture visuelle trop volumineuse.".to_string())?;
    if screenshot.bytes.len() != expected {
        return Err("Capture visuelle Iced incomplète.".into());
    }
    let mut ppm = format!(
        "P6\n{} {}\n255\n",
        screenshot.size.width, screenshot.size.height
    )
    .into_bytes();
    ppm.reserve(usize::try_from(pixel_count.saturating_mul(3)).unwrap_or_default());
    for rgba in screenshot.bytes.chunks_exact(4) {
        ppm.extend_from_slice(&rgba[..3]);
    }
    std::fs::write(path, ppm)
        .map_err(|error| format!("Impossible d'enregistrer la capture visuelle : {error}"))
}
