//! Frontière IPC avec événements globaux `ia-progression`.

use crate::app::state::AppState;
use crate::core::errors::AppResult;
use crate::features::ia::domain::{
    AnalyseCvImporte, AnalyseOffre, DemandeAnalyseCv, DemandeGenerationCv, DemandeImportProfil,
    DemandeLettre, GenerationCv, ProfilExtrait, ProgressionIa,
};
use tauri::{AppHandle, Emitter, State};

fn notifier(app: AppHandle) -> impl Fn(ProgressionIa) {
    move |progression| {
        if let Err(error) = app.emit("ia-progression", progression) {
            tracing::warn!(%error, "progression IA non émise");
        }
    }
}

#[tauri::command]
pub async fn ia_analyser_offre(
    state: State<'_, AppState>,
    texte: String,
) -> AppResult<AnalyseOffre> {
    state.ia.analyser_offre(texte).await
}

#[tauri::command]
pub async fn ia_generer_cv(
    app: AppHandle,
    state: State<'_, AppState>,
    demande: DemandeGenerationCv,
) -> AppResult<GenerationCv> {
    state.ia.generer_cv(demande, notifier(app)).await
}

#[tauri::command]
pub async fn ia_generer_lettre(
    app: AppHandle,
    state: State<'_, AppState>,
    demande: DemandeLettre,
) -> AppResult<String> {
    state.ia.generer_lettre(demande, notifier(app)).await
}

#[tauri::command]
pub async fn ia_analyser_cv(
    app: AppHandle,
    state: State<'_, AppState>,
    demande: DemandeAnalyseCv,
) -> AppResult<AnalyseCvImporte> {
    state.ia.analyser_cv_importe(demande, notifier(app)).await
}

#[tauri::command]
pub async fn ia_importer_profil(
    app: AppHandle,
    state: State<'_, AppState>,
    demande: DemandeImportProfil,
) -> AppResult<ProfilExtrait> {
    state.ia.importer_profil(demande, notifier(app)).await
}

#[tauri::command]
pub async fn ia_annuler(state: State<'_, AppState>, generation_id: String) -> AppResult<()> {
    state.ia.annuler(&generation_id);
    Ok(())
}
