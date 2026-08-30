//! Construction et lancement de l'application Tauri.

use crate::app::state::AppState;
use crate::features::ai::presentation::commands as ai;
use crate::features::analytics::presentation::commands as analytics;
use crate::features::applications::presentation::commands as applications;
use crate::features::companies::presentation::commands as companies;
use crate::features::contacts::presentation::commands as contacts;
use crate::features::documents::presentation::commands as documents;
use crate::features::followups::presentation::commands as followups;
use crate::features::interviews::presentation::commands as interviews;
use crate::features::profile::presentation::commands as profile;
use crate::features::sectors::presentation::commands as sectors;
use crate::features::settings::presentation::commands as settings;

/// Démarre Candilog : journal, état applicatif, plugins, commandes.
///
/// Sans base de données lisible il n'y a rien à afficher : l'application s'arrête en
/// journalisant la cause plutôt que d'ouvrir une fenêtre vide dont l'utilisateur ne pourrait
/// rien tirer. C'est le seul point du programme où un arrêt est le bon comportement.
pub fn run() {
    let _guard = crate::core::logging::init();

    let state = match AppState::persistent() {
        Ok(state) => state,
        Err(error) => {
            tracing::error!(%error, "état applicatif non initialisable");
            eprintln!(
                "Candilog n'a pas pu ouvrir ses données : {}",
                error.user_message()
            );
            std::process::exit(1);
        }
    };

    if let Err(error) = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            analytics::analytics_dashboard,
            analytics::analytics_load,
            analytics::analytics_export_csv,
            applications::applications_list_page,
            applications::applications_breakdown,
            applications::applications_get,
            applications::applications_create,
            applications::applications_update,
            applications::applications_change_status,
            applications::applications_delete,
            applications::applications_export_csv,
            companies::companies_list,
            companies::companies_list_page,
            companies::companies_list_types,
            companies::companies_get,
            companies::companies_create,
            companies::companies_update,
            companies::companies_delete,
            contacts::contacts_list,
            contacts::contacts_list_page,
            contacts::contacts_get,
            contacts::contacts_create,
            contacts::contacts_update,
            contacts::contacts_delete,
            documents::documents_resume_list,
            documents::documents_resume_list_page,
            documents::documents_resume_get,
            documents::documents_resume_save,
            documents::documents_resume_delete,
            documents::documents_resume_export_pdf,
            documents::documents_cover_letters_list,
            documents::documents_cover_letters_list_page,
            documents::documents_cover_letter_get,
            documents::documents_cover_letter_save,
            documents::documents_cover_letter_delete,
            documents::documents_cover_letter_export_pdf,
            interviews::interviews_list_between,
            interviews::interviews_get,
            interviews::interviews_save,
            interviews::interviews_delete,
            ai::ai_analyze_listing,
            ai::ai_generate_resume,
            ai::ai_generate_cover_letter,
            ai::ai_analyze_resume,
            ai::ai_import_profile,
            ai::ai_cancel,
            settings::settings_load,
            settings::settings_save,
            settings::settings_clear_api_key,
            settings::settings_test_connection,
            settings::settings_list_models,
            settings::settings_clear_ai_cache,
            settings::settings_export,
            settings::settings_restore,
            settings::settings_reset,
            settings::settings_check_update,
            settings::settings_download_update,
            settings::settings_about,
            profile::profile_load,
            profile::profile_save,
            profile::profile_apply_import,
            followups::follow_ups_list_between,
            followups::follow_ups_create,
            followups::follow_ups_update,
            followups::follow_ups_delete,
            sectors::sectors_list,
        ])
        .run(tauri::generate_context!())
    {
        // `run` n'échoue qu'à l'initialisation du moteur de rendu système : rien que
        // l'application puisse corriger, mais la cause doit rester dans le journal.
        tracing::error!(%error, "démarrage de la fenêtre impossible");
        std::process::exit(1);
    }
}
