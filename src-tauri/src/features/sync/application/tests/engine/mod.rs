//! Scénarios multi-ordinateurs du moteur : plusieurs bases en mémoire partagent un espace
//! distant simulé. Aucun compte réel n'est utilisé.

use super::*;
use crate::core::database::{open_pool, run_local_migrations, SqlitePool};
use crate::features::sync::domain::model::SyncProviderKind;
use crate::features::sync::infrastructure::memory::MemoryStore;

mod test_concurrence_et_conflits;
mod test_coupures_et_reprises;
mod test_documents_et_fichiers;
mod test_integrite_et_securite;
mod test_propagation;

pub(super) const SPACE: &str = "22222222-2222-4222-8222-222222222222";
pub(super) const TS: &str = "2026-01-01T00:00:00+00:00";

/// Un espace distant partagé.
pub(super) struct Space {
    pub remote: MemoryStore,
}

impl Space {
    pub fn new() -> Self {
        Self {
            remote: MemoryStore::default(),
        }
    }

    /// Ordinateur rattaché à l'espace, données locales publiées au premier lot.
    pub fn computer(&self) -> Computer {
        let computer = Computer::detached(self);
        computer.attach(true);
        computer
    }
}

/// Une installation de Candilog : base, dossier des photos, moteur et session.
pub(super) struct Computer {
    pub pool: SqlitePool,
    pub engine: SyncEngine,
    pub session: Session,
    pub photos: tempfile::TempDir,
}

impl Computer {
    pub fn detached(space: &Space) -> Self {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        let photos = tempfile::tempdir().unwrap();
        let installation_id = uuid::Uuid::new_v4().to_string();
        let session = Session {
            remote: Arc::new(space.remote.clone()),
            space_id: SPACE.into(),
            installation_id,
        };
        Self {
            engine: SyncEngine::new(
                SqliteSyncStore::new(pool.clone()),
                photos.path().to_path_buf(),
            ),
            pool,
            session,
            photos,
        }
    }

    pub fn config(&self) -> SyncConfig {
        SyncConfig {
            provider: SyncProviderKind::Folder,
            folder: None,
            space_id: Some(SPACE.into()),
            installation_id: Some(self.session.installation_id.clone()),
            auto_sync: true,
        }
    }

    pub fn attach(&self, publish: bool) {
        self.engine.store().attach(&self.config(), publish).unwrap();
    }

    pub async fn sync(&self) -> CycleReport {
        self.engine.cycle(&self.session).await
    }

    pub async fn sync_ok(&self) -> CycleReport {
        let report = self.sync().await;
        assert_eq!(report.failure, None, "cycle en échec : {report:?}");
        report
    }

    pub fn exec(&self, sql: &str) {
        self.pool.get().unwrap().execute_batch(sql).unwrap();
    }

    pub fn text(&self, sql: &str) -> Option<String> {
        use rusqlite::OptionalExtension;
        self.pool
            .get()
            .unwrap()
            .query_row(sql, [], |row| row.get::<_, Option<String>>(0))
            .optional()
            .unwrap()
            .flatten()
    }

    pub fn count(&self, sql: &str) -> i64 {
        self.pool
            .get()
            .unwrap()
            .query_row(sql, [], |row| row.get(0))
            .unwrap()
    }

    pub fn add_company(&self, id: &str, name: &str) {
        self.exec(&format!(
            "INSERT INTO companies (id, name, created_at, updated_at) VALUES ('{id}', '{name}', '{TS}', '{TS}')"
        ));
    }

    pub fn add_application(&self, id: &str, company: &str, title: &str) {
        self.exec(&format!(
            "INSERT INTO applications (id, company_id, job_title, contract_type_code, status, sent_date, created_at, updated_at)
             VALUES ('{id}', '{company}', '{title}', 'CDI', 'EN_ATTENTE', '2026-01-01', '{TS}', '{TS}')"
        ));
    }

    pub fn add_follow_up(&self, id: &str, application: &str) {
        self.exec(&format!(
            "INSERT INTO follow_ups (id, application_id, follow_up_date, created_at) VALUES ('{id}', '{application}', '2026-02-01', '{TS}')"
        ));
    }

    pub fn conflicts(&self) -> Vec<crate::features::sync::infrastructure::apply::StoredConflict> {
        self.engine.store().conflicts().unwrap()
    }
}
