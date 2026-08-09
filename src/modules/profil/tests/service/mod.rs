//! Helpers communs et déclaration des cas de test.
use super::*;
use crate::shared::profile::{Experience, PersonalInfo};
use std::sync::Mutex;

struct StubRepo {
    store: Mutex<Option<Profile>>,
}
impl ProfilRepository for StubRepo {
    fn get(&self) -> AppResult<Profile> {
        Ok(self.store.lock().unwrap().clone().unwrap_or_default())
    }
    fn upsert(&self, p: &Profile) -> AppResult<Profile> {
        *self.store.lock().unwrap() = Some(p.clone());
        Ok(p.clone())
    }
}
fn service() -> ProfilService<StubRepo> {
    ProfilService::new(StubRepo {
        store: Mutex::new(None),
    })
}

mod test_update_email_invalide_retourne_erreur;
mod test_update_experience_sans_titre_retourne_erreur;
mod test_update_profil_valide_persiste;
