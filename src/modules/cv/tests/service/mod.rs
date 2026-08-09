//! Helpers communs et déclaration des cas de test.
use super::*;
use std::sync::Mutex;

#[derive(Default)]
struct MockRepo {
    created: Mutex<Vec<(String, Value)>>,
    deleted: Mutex<Vec<Uuid>>,
}
impl CvVersionRepository for MockRepo {
    fn create(&self, name: &str, content: &Value) -> AppResult<CvVersion> {
        self.created
            .lock()
            .unwrap()
            .push((name.to_string(), content.clone()));
        Ok(CvVersion {
            id: Uuid::nil(),
            name: name.to_string(),
            content: content.clone(),
            created_at: "now".into(),
        })
    }
    fn list(&self) -> AppResult<Vec<CvVersionSummary>> {
        Ok(vec![])
    }
    fn get(&self, id: Uuid) -> AppResult<CvVersion> {
        Ok(CvVersion {
            id,
            name: "x".into(),
            content: Value::Null,
            created_at: "now".into(),
        })
    }
    fn delete(&self, id: Uuid) -> AppResult<()> {
        self.deleted.lock().unwrap().push(id);
        Ok(())
    }
}

mod test_delete_delegue_au_depot;
mod test_save_nom_trop_long_retourne_validation;
mod test_save_nom_valide_delegue_au_depot_avec_nom_trim;
mod test_save_nom_vide_retourne_validation;
