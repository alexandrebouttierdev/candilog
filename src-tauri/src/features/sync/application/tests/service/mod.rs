//! Parcours de bout en bout du service, avec le fournisseur « dossier » réel sur disque et
//! un coffre en mémoire. Aucun compte Google ou Microsoft n'est utilisé.

use super::*;
use crate::core::database::{open_pool, run_local_migrations, SqlitePool};
use crate::features::sync::infrastructure::secrets::MemorySecrets;

const PASSPHRASE: &str = "phrase secrète de test";
const TS: &str = "2026-01-01T00:00:00+00:00";

struct Install {
    pool: SqlitePool,
    service: Arc<SyncService>,
    secrets: Arc<MemorySecrets>,
    data: tempfile::TempDir,
}

impl Install {
    fn new() -> Self {
        let pool = open_pool(None).unwrap();
        run_local_migrations(&pool).unwrap();
        let data = tempfile::tempdir().unwrap();
        let secrets = Arc::new(MemorySecrets::default());
        let service = Arc::new(SyncService::new(
            SqliteSyncStore::new(pool.clone()),
            data.path().join("photos"),
            data.path().join("exports"),
            secrets.clone(),
        ));
        Self {
            pool,
            service,
            secrets,
            data,
        }
    }

    fn exec(&self, sql: &str) {
        self.pool.get().unwrap().execute_batch(sql).unwrap();
    }

    fn count(&self, sql: &str) -> i64 {
        self.pool
            .get()
            .unwrap()
            .query_row(sql, [], |row| row.get(0))
            .unwrap()
    }

    fn add_company(&self, id: &str) {
        self.exec(&format!(
            "INSERT INTO companies (id, name, created_at, updated_at) VALUES ('{id}', 'Entreprise {id}', '{TS}', '{TS}')"
        ));
    }

    async fn choose(&self, folder: &Path) -> RemoteProbe {
        self.service
            .choose_folder(folder.to_path_buf(), self.data.path())
            .await
            .unwrap()
    }

    async fn setup(
        &self,
        choice: SetupChoice,
        passphrase: Option<&str>,
        recovery: Option<&str>,
    ) -> AppResult<SetupOutcome> {
        self.service
            .setup(SetupRequest {
                choice,
                passphrase: passphrase.map(str::to_owned),
                recovery_key: recovery.map(str::to_owned),
            })
            .await
    }

    async fn create_space(&self, folder: &Path) -> SetupOutcome {
        self.choose(folder).await;
        self.setup(SetupChoice::CreateSpace, Some(PASSPHRASE), None)
            .await
            .unwrap()
    }

    fn phase(&self) -> SyncPhase {
        self.service.status().unwrap().phase
    }
}

#[tokio::test]
async fn le_mode_local_est_celui_par_defaut() {
    let install = Install::new();
    let status = install.service.status().unwrap();
    assert_eq!(status.provider, SyncProviderKind::Local);
    assert_eq!(status.phase, SyncPhase::NotConfigured);
    install.add_company("c1");
    assert_eq!(install.count("SELECT count(*) FROM sync_outbox"), 0);
    install.service.run_cycle().await;
    assert_eq!(install.phase(), SyncPhase::NotConfigured);
}

#[tokio::test]
async fn creer_un_espace_publie_les_donnees_locales_chiffrees() {
    let folder = tempfile::tempdir().unwrap();
    let install = Install::new();
    install.add_company("c1");
    let probe = install.choose(folder.path()).await;
    assert!(!probe.space_exists);
    assert!(probe.local_has_data);

    let outcome = install
        .setup(SetupChoice::CreateSpace, Some(PASSPHRASE), None)
        .await
        .unwrap();
    let recovery = outcome.recovery_key.unwrap();
    assert_eq!(
        recovery.split('-').count(),
        13,
        "clé de récupération lisible par groupes"
    );
    assert_eq!(install.phase(), SyncPhase::Pending);

    install.service.run_cycle().await;
    let status = install.service.status().unwrap();
    assert_eq!(status.phase, SyncPhase::UpToDate, "{status:?}");
    assert!(status.last_success_at.is_some());
    assert_eq!(status.pending_changes, 0);
    let space = folder.path().join("candilog-sync");
    assert!(space.join("space.json").is_file());
    let manifest = std::fs::read_to_string(space.join("space.json")).unwrap();
    assert!(
        !manifest.contains("Entreprise"),
        "le manifeste ne porte aucune donnée"
    );
}

#[tokio::test]
async fn une_phrase_trop_courte_ne_cree_rien() {
    let folder = tempfile::tempdir().unwrap();
    let install = Install::new();
    install.choose(folder.path()).await;
    assert!(install
        .setup(SetupChoice::CreateSpace, Some("court"), None)
        .await
        .is_err());
    assert_eq!(install.phase(), SyncPhase::NotConfigured);
    assert!(!folder.path().join("candilog-sync/space.json").exists());
}

#[tokio::test]
async fn rejoindre_avec_une_phrase_incorrecte_ne_rattache_rien() {
    let folder = tempfile::tempdir().unwrap();
    let first = Install::new();
    first.create_space(folder.path()).await;
    let second = Install::new();
    assert!(second.choose(folder.path()).await.space_exists);
    let error = second
        .setup(
            SetupChoice::JoinMerge,
            Some("une autre phrase secrète"),
            None,
        )
        .await
        .unwrap_err();
    assert!(error.user_message().contains("incorrecte"));
    assert_eq!(second.phase(), SyncPhase::NotConfigured);
}

#[tokio::test]
async fn deux_ordinateurs_se_synchronisent_par_un_dossier() {
    let folder = tempfile::tempdir().unwrap();
    let first = Install::new();
    first.add_company("c1");
    first.create_space(folder.path()).await;
    first.service.run_cycle().await;

    let second = Install::new();
    second.add_company("c2");
    second.choose(folder.path()).await;
    second
        .setup(SetupChoice::JoinMerge, Some(PASSPHRASE), None)
        .await
        .unwrap();
    second.service.run_cycle().await;
    first.service.run_cycle().await;

    for install in [&first, &second] {
        assert_eq!(install.count("SELECT count(*) FROM companies"), 2);
        assert_eq!(install.phase(), SyncPhase::UpToDate);
    }
    assert_eq!(second.service.status().unwrap().computers, 2);
}

#[tokio::test]
async fn la_cle_de_recuperation_permet_de_rejoindre() {
    let folder = tempfile::tempdir().unwrap();
    let first = Install::new();
    first.add_company("c1");
    let recovery = first
        .create_space(folder.path())
        .await
        .recovery_key
        .unwrap();
    first.service.run_cycle().await;

    let second = Install::new();
    second.choose(folder.path()).await;
    second
        .setup(SetupChoice::JoinMerge, None, Some(&recovery))
        .await
        .unwrap();
    second.service.run_cycle().await;
    assert_eq!(second.count("SELECT count(*) FROM companies"), 1);
}

#[tokio::test]
async fn recuperer_l_espace_sauvegarde_puis_remplace_les_donnees_locales() {
    let folder = tempfile::tempdir().unwrap();
    let first = Install::new();
    first.add_company("distante");
    first.create_space(folder.path()).await;
    first.service.run_cycle().await;

    let second = Install::new();
    second.add_company("locale");
    second.choose(folder.path()).await;
    let outcome = second
        .setup(SetupChoice::JoinReplaceLocal, Some(PASSPHRASE), None)
        .await
        .unwrap();
    let backup_file = outcome.backup_file.unwrap();
    assert!(
        Path::new(&backup_file).is_file(),
        "sauvegarde prise avant le remplacement"
    );
    crate::core::backup::validate(Path::new(&backup_file)).unwrap();
    assert_eq!(
        second.count("SELECT count(*) FROM companies WHERE id = 'locale'"),
        0
    );
    assert_eq!(
        second.count("SELECT count(*) FROM companies WHERE id = 'distante'"),
        1
    );
}

#[tokio::test]
async fn creer_un_second_espace_au_meme_endroit_est_refuse() {
    let folder = tempfile::tempdir().unwrap();
    Install::new().create_space(folder.path()).await;
    let second = Install::new();
    second.choose(folder.path()).await;
    assert!(matches!(
        second
            .setup(SetupChoice::CreateSpace, Some(PASSPHRASE), None)
            .await,
        Err(AppError::Validation(_))
    ));
}

#[tokio::test]
async fn rejoindre_un_emplacement_vide_est_refuse() {
    let folder = tempfile::tempdir().unwrap();
    let install = Install::new();
    install.choose(folder.path()).await;
    assert!(matches!(
        install
            .setup(SetupChoice::JoinMerge, Some(PASSPHRASE), None)
            .await,
        Err(AppError::Validation(_))
    ));
}

#[tokio::test]
async fn se_deconnecter_garde_les_donnees_locales_et_l_espace() {
    let folder = tempfile::tempdir().unwrap();
    let install = Install::new();
    install.add_company("c1");
    install.create_space(folder.path()).await;
    install.service.run_cycle().await;
    let files_before = std::fs::read_dir(folder.path().join("candilog-sync/changes"))
        .unwrap()
        .count();

    install.service.disconnect().await.unwrap();
    assert_eq!(install.phase(), SyncPhase::NotConfigured);
    assert_eq!(
        install.count("SELECT count(*) FROM companies"),
        1,
        "rien d'effacé localement"
    );
    assert_eq!(install.count("SELECT count(*) FROM sync_records"), 0);
    assert!(
        install.secrets.0.lock().unwrap().is_empty(),
        "clé retirée du coffre"
    );
    assert_eq!(
        std::fs::read_dir(folder.path().join("candilog-sync/changes"))
            .unwrap()
            .count(),
        files_before,
        "rien d'effacé à distance"
    );
    // Les modifications suivantes ne sont plus journalisées.
    install.add_company("c2");
    assert_eq!(install.count("SELECT count(*) FROM sync_outbox"), 0);
}

#[tokio::test]
async fn changer_de_fournisseur_copie_les_donnees_sans_toucher_l_ancien_espace() {
    let old = tempfile::tempdir().unwrap();
    let new = tempfile::tempdir().unwrap();
    let install = Install::new();
    install.add_company("c1");
    install.create_space(old.path()).await;
    install.service.run_cycle().await;
    let old_space = std::fs::read_to_string(old.path().join("candilog-sync/space.json")).unwrap();

    install.service.disconnect().await.unwrap();
    install.create_space(new.path()).await;
    install.service.run_cycle().await;

    let other = Install::new();
    other.choose(new.path()).await;
    other
        .setup(SetupChoice::JoinMerge, Some(PASSPHRASE), None)
        .await
        .unwrap();
    other.service.run_cycle().await;
    assert_eq!(other.count("SELECT count(*) FROM companies"), 1);
    assert_eq!(
        std::fs::read_to_string(old.path().join("candilog-sync/space.json")).unwrap(),
        old_space
    );
}

#[tokio::test]
async fn un_dossier_dans_les_donnees_de_candilog_est_refuse() {
    let install = Install::new();
    let inside = install.data.path().join("photos");
    std::fs::create_dir_all(&inside).unwrap();
    assert!(install
        .service
        .choose_folder(inside, install.data.path())
        .await
        .is_err());
}

#[tokio::test]
async fn un_dossier_demonte_est_signale_sans_perdre_les_modifications() {
    let folder = tempfile::tempdir().unwrap();
    let install = Install::new();
    install.create_space(folder.path()).await;
    install.service.run_cycle().await;
    std::fs::remove_dir_all(folder.path().join("candilog-sync")).unwrap();
    install.add_company("c1");
    install.service.run_cycle().await;
    let status = install.service.status().unwrap();
    assert_eq!(status.phase, SyncPhase::TemporaryError);
    assert!(status.message.unwrap().contains("inaccessible"));
    assert!(status.next_attempt_at.is_some(), "reprise programmée");
    assert_eq!(status.pending_changes, 1);
    assert!(
        !folder.path().join("candilog-sync").exists(),
        "le dossier n'est pas recréé vide"
    );
}

#[tokio::test]
async fn une_cle_absente_du_coffre_se_retrouve_avec_la_phrase_secrete() {
    let folder = tempfile::tempdir().unwrap();
    let install = Install::new();
    install.add_company("c1");
    install.create_space(folder.path()).await;
    install.service.run_cycle().await;
    let before = install.service.config().unwrap().installation_id;

    install.secrets.0.lock().unwrap().clear();
    install.service.run_cycle().await;
    assert_eq!(install.phase(), SyncPhase::KeyRequired);

    assert!(install
        .service
        .unlock_space(Some("mauvaise phrase secrète".into()), None)
        .await
        .is_err());
    install
        .service
        .unlock_space(Some(PASSPHRASE.into()), None)
        .await
        .unwrap();
    install.service.run_cycle().await;
    assert_eq!(install.phase(), SyncPhase::UpToDate);
    assert_ne!(
        install.service.config().unwrap().installation_id,
        before,
        "nouvelle identité"
    );
}

#[tokio::test]
async fn suspendre_la_synchronisation_conserve_les_modifications() {
    let folder = tempfile::tempdir().unwrap();
    let install = Install::new();
    install.create_space(folder.path()).await;
    install.service.set_auto(false).unwrap();
    install.add_company("c1");
    let status = install.service.status().unwrap();
    assert_eq!(status.phase, SyncPhase::Disabled);
    assert_eq!(status.pending_changes, 1);
    assert!(!install.service.due(false), "aucun cycle automatique");
    assert!(
        install.service.due(true),
        "la synchronisation manuelle reste possible"
    );
}

#[tokio::test]
async fn les_conflits_sont_presentes_en_francais() {
    let folder = tempfile::tempdir().unwrap();
    let first = Install::new();
    first.add_company("c1");
    first.create_space(folder.path()).await;
    first.service.run_cycle().await;
    let second = Install::new();
    second.choose(folder.path()).await;
    second
        .setup(SetupChoice::JoinMerge, Some(PASSPHRASE), None)
        .await
        .unwrap();
    second.service.run_cycle().await;

    first.exec("UPDATE companies SET city = 'Lyon' WHERE id = 'c1'");
    second.exec("UPDATE companies SET city = 'Paris' WHERE id = 'c1'");
    first.service.run_cycle().await;
    second.service.run_cycle().await;

    assert_eq!(second.phase(), SyncPhase::Conflict);
    let conflicts = second.service.conflicts().unwrap();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].entity_label, "Entreprise");
    assert_eq!(conflicts[0].record_label, "Entreprise c1");
    assert_eq!(conflicts[0].fields[0].label, "Ville");
    assert_eq!(conflicts[0].fields[0].local.as_deref(), Some("Paris"));
    assert_eq!(conflicts[0].fields[0].remote.as_deref(), Some("Lyon"));

    assert_eq!(
        second
            .service
            .resolve_all(ConflictChoice::TakeRemote)
            .unwrap(),
        1
    );
    second.service.run_cycle().await;
    first.service.run_cycle().await;
    assert_eq!(
        first.count("SELECT count(*) FROM companies WHERE city = 'Lyon'"),
        1
    );
    assert_eq!(
        second.count("SELECT count(*) FROM companies WHERE city = 'Lyon'"),
        1
    );
}

#[tokio::test]
async fn sans_application_oauth_les_fournisseurs_cloud_sont_indisponibles() {
    let availability = SyncService::availability();
    // Les tests ne sont jamais compilés avec de vrais identifiants.
    if option_env!("CANDILOG_GOOGLE_CLIENT_ID").is_none() {
        assert!(!availability.google_drive);
        let install = Install::new();
        let open = |_: &str| Ok(());
        let error = install
            .service
            .connect_cloud(SyncProviderKind::GoogleDrive, &open)
            .await
            .unwrap_err();
        assert!(error.user_message().contains("n'est pas disponible"));
    }
}
