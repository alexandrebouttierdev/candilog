//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::shared::llm::LlmConfig;
use std::sync::Mutex;

struct StubRepo {
    store: Mutex<Option<AppSettings>>,
}
impl SettingsRepository for StubRepo {
    fn get(&self) -> AppResult<AppSettings> {
        Ok(self.store.lock().unwrap().clone().unwrap_or_default())
    }
    fn upsert(&self, s: &AppSettings) -> AppResult<AppSettings> {
        *self.store.lock().unwrap() = Some(s.clone());
        Ok(s.clone())
    }
}
fn service() -> SettingsService<StubRepo> {
    SettingsService::new(StubRepo {
        store: Mutex::new(None),
    })
}

mod test_update_config_ollama_valide_persiste;
mod test_update_custom_sans_endpoint_retourne_erreur;
mod test_update_provider_cloud_sans_cle_retourne_erreur;
mod test_update_temperature_hors_bornes_retourne_erreur;
