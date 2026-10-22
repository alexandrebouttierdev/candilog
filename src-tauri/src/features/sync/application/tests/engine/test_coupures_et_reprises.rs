//! Coupures réseau, réponses perdues et redémarrages : rien ne se perd, rien ne se double.

use super::*;
use crate::features::sync::domain::remote::RemoteError;

#[tokio::test]
async fn hors_ligne_les_modifications_attendent_puis_partent() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    space.remote.set_offline(true);
    a.add_company("c1", "Acme");
    let report = a.sync().await;
    assert!(matches!(
        report.failure,
        Some(SyncFailure::Remote(RemoteError::Offline(_)))
    ));
    assert_eq!(
        a.engine.store().counts().unwrap().pending,
        1,
        "toujours en attente"
    );

    space.remote.set_offline(false);
    a.sync_ok().await;
    b.sync_ok().await;
    assert_eq!(b.count("SELECT count(*) FROM companies"), 1);
}

#[tokio::test]
async fn les_operations_en_attente_survivent_a_un_redemarrage() {
    let space = Space::new();
    let a = space.computer();
    space.remote.set_offline(true);
    a.add_company("c1", "Acme");
    let _ = a.sync().await;

    // « Redémarrage » : nouveau moteur sur la même base.
    let restarted = SyncEngine::new(
        SqliteSyncStore::new(a.pool.clone()),
        a.photos.path().to_path_buf(),
    );
    space.remote.set_offline(false);
    let report = restarted.cycle(&a.session).await;
    assert_eq!(report.failure, None);
    assert_eq!(report.published, 1);
    let b = space.computer();
    b.sync_ok().await;
    assert_eq!(b.count("SELECT count(*) FROM companies"), 1);
}

#[tokio::test]
async fn une_reponse_perdue_apres_envoi_ne_double_rien() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    a.add_company("c1", "Acme");
    space.remote.lose_next_acks(1);
    let report = a.sync().await;
    assert!(report.failure.is_some(), "l'envoi semble avoir échoué");
    assert_eq!(a.engine.store().counts().unwrap().pending, 1);

    // Le même lot, sous le même nom, est reconnu comme déjà écrit.
    a.add_company("c2", "Globex");
    a.sync_ok().await;
    assert_eq!(a.engine.store().counts().unwrap().pending, 0);
    let batches: Vec<_> = space
        .remote
        .names()
        .into_iter()
        .filter(|name| name.starts_with("v2/changes/"))
        .collect();
    assert_eq!(batches.len(), 2, "un lot pour c1, un pour c2 : {batches:?}");

    b.sync_ok().await;
    assert_eq!(b.count("SELECT count(*) FROM companies"), 2);
}

#[tokio::test]
async fn un_lot_en_attente_n_est_jamais_reecrit_avec_un_autre_contenu() {
    let space = Space::new();
    let a = space.computer();
    a.add_company("c1", "Acme");
    space.remote.set_offline(true);
    let _ = a.sync().await;
    let pending = a.engine.store().pending_batch().unwrap().unwrap();
    // Une modification arrivée entre-temps part dans le lot suivant.
    a.exec("UPDATE companies SET name = 'Acme SA' WHERE id = 'c1'");
    space.remote.set_offline(false);
    a.sync_ok().await;
    assert_eq!(space.remote.raw(&pending.name).unwrap(), pending.payload);
    assert_eq!(a.engine.store().counts().unwrap().pending, 0);
}

#[tokio::test]
async fn un_lot_manquant_bloque_la_lecture_sans_jamais_le_sauter() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    a.add_company("c1", "Acme");
    a.sync_ok().await;
    a.add_company("c2", "Globex");
    a.sync_ok().await;
    // Le premier lot n'est pas encore arrivé (outil tiers en retard).
    let first = crate::features::sync::domain::protocol::batch_name(&a.session.installation_id, 1);
    let saved = space.remote.raw(&first).unwrap();
    space.remote.remove(&first);

    let report = b.sync().await;
    assert!(matches!(report.failure, Some(SyncFailure::Integrity(_))));
    assert_eq!(
        b.count("SELECT count(*) FROM companies"),
        0,
        "le lot 2 n'est pas appliqué seul"
    );

    space.remote.overwrite(&first, saved);
    b.sync_ok().await;
    assert_eq!(b.count("SELECT count(*) FROM companies"), 2);
}

#[tokio::test]
async fn une_base_copiee_d_un_autre_ordinateur_change_d_identite() {
    let space = Space::new();
    let a = space.computer();
    a.add_company("c1", "Acme");
    a.sync_ok().await;

    // Copie de la base de A sur un autre poste, qui publie sous la même identité.
    let copy = Computer::detached(&space);
    let mut session = copy.session.clone();
    session.installation_id = a.session.installation_id.clone();
    copy.engine
        .store()
        .attach(
            &{
                let mut c = copy.config();
                c.installation_id = Some(session.installation_id.clone());
                c
            },
            true,
        )
        .unwrap();
    copy.add_company("c-copie", "Créée sur la copie");
    let report = copy.engine.cycle(&session).await;
    assert!(report.renewed_installation.is_some());
    // Le lot de A n'a pas été remplacé.
    let b = space.computer();
    b.sync().await;
    assert_eq!(b.count("SELECT count(*) FROM companies WHERE id = 'c1'"), 1);
}
