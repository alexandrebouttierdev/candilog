//! Construction et lancement de l'application Tauri.

use crate::app::state::AppState;

/// Démarre Candilog : journal, état applicatif, plugins, commandes.
///
/// Sans base de données lisible il n'y a rien à afficher : l'application s'arrête en
/// journalisant la cause plutôt que d'ouvrir une fenêtre vide dont l'utilisateur ne pourrait
/// rien tirer. C'est le seul point du programme où un arrêt est le bon comportement.
pub fn run() {
    let _garde = crate::core::logging::initialiser();

    let state = match AppState::persistent() {
        Ok(state) => state,
        Err(error) => {
            tracing::error!(%error, "état applicatif non initialisable");
            eprintln!(
                "Candilog n'a pas pu ouvrir ses données : {}",
                error.message_utilisateur()
            );
            std::process::exit(1);
        }
    };

    if let Err(error) = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
    {
        // `run` n'échoue qu'à l'initialisation du moteur de rendu système : rien que
        // l'application puisse corriger, mais la cause doit rester dans le journal.
        tracing::error!(%error, "démarrage de la fenêtre impossible");
        std::process::exit(1);
    }
}
