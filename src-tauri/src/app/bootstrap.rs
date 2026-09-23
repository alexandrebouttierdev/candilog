//! Construction et lancement de l'application Tauri.

use crate::app::state::AppState;
use crate::features::ai::presentation::{
    commands as ai, managed_ollama_commands as managed_ollama, system_commands as system_resources,
};
use crate::features::analytics::presentation::commands as analytics;
use crate::features::applications::presentation::commands as applications;
use crate::features::companies::presentation::commands as companies;
use crate::features::contacts::presentation::commands as contacts;
use crate::features::documents::presentation::commands as documents;
use crate::features::followups::presentation::commands as followups;
use crate::features::interviews::presentation::commands as interviews;
use crate::features::profile::presentation::commands as profile;
use crate::features::referentials::presentation::commands as referentials;
use crate::features::settings::presentation::commands as settings;
use tauri::Manager;

pub fn run() {
    let _guard = crate::core::logging::init();

    let state = match AppState::persistent() {
        Ok(state) => state,
        Err(error) => {
            tracing::error!(%error, "état applicatif non initialisable");
            signaler_demarrage_impossible(&error.user_message());
            std::process::exit(1);
        }
    };

    if let Err(error) = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(state)
        .on_window_event(|window, event| {
            if window.label() == "main" && matches!(event, tauri::WindowEvent::Destroyed) {
                let state = window.state::<AppState>();
                state.managed_ollama.shutdown();
            }
        })
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
            applications::applications_deletion_impact,
            applications::applications_status_history,
            applications::applications_duplicate,
            applications::applications_export_csv,
            companies::companies_list,
            companies::companies_list_page,
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
            documents::documents_read_clipboard,
            documents::documents_resume_list_page,
            documents::documents_resume_get,
            documents::documents_resume_save,
            documents::documents_resume_delete,
            documents::documents_resume_prepare,
            documents::documents_resume_recalculate,
            documents::documents_resume_apply_proposal,
            documents::documents_resume_reject_proposal,
            documents::documents_resume_export_pdf,
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
            ai::ai_correct_french,
            ai::ai_analyze_resume,
            ai::ai_select_resume_file,
            ai::ai_import_profile,
            ai::ai_active_model_capabilities,
            ai::ai_cancel,
            managed_ollama::get_managed_ollama_status,
            managed_ollama::install_managed_ollama_model,
            managed_ollama::cancel_managed_ollama_download,
            managed_ollama::remove_managed_ollama_model,
            managed_ollama::activate_managed_ollama_model,
            managed_ollama::run_user_cv_benchmark,
            system_resources::system_resource_snapshot,
            settings::settings_load,
            settings::settings_save,
            settings::settings_clear_api_key,
            settings::settings_test_connection,
            settings::settings_list_models,
            settings::settings_export,
            settings::settings_restore,
            settings::settings_reset,
            settings::settings_check_update,
            settings::settings_download_update,
            settings::settings_about,
            settings::open_external_url,
            profile::profile_load,
            profile::profile_save,
            profile::profile_apply_import,
            profile::profile_add_skill,
            profile::profile_set_photo,
            profile::profile_remove_photo,
            profile::profile_photo,
            profile::profile_reset,
            followups::follow_ups_list_between,
            followups::follow_ups_create,
            followups::follow_ups_update,
            followups::follow_ups_delete,
            referentials::referentials_load,
        ])
        .run(tauri::generate_context!())
    {
        tracing::error!(%error, "démarrage de la fenêtre impossible");
        std::process::exit(1);
    }
    crate::core::logging::cloturer_session();
}

fn signaler_demarrage_impossible(message: &str) {
    eprintln!("Candilog n'a pas pu ouvrir ses données : {message}");
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("Candilog n'a pas pu démarrer")
        .set_description(message)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}
