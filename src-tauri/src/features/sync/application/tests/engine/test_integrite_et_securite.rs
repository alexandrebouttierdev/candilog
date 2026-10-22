//! Chiffrement, clé incorrecte, espace altéré ou incompatible.

use super::*;
use crate::features::sync::domain::protocol::{batch_name, ChangeBatch, FORMAT_VERSION};

#[tokio::test]
async fn rien_ne_part_en_clair() {
    let space = Space::new();
    let a = space.computer();
    a.add_company("c1", "Entreprise Confidentielle");
    a.exec("UPDATE companies SET notes = 'salaire souhaité 45000' WHERE id = 'c1'");
    a.sync_ok().await;
    for name in space.remote.names() {
        let bytes = space.remote.raw(&name).unwrap();
        for secret in [&b"Confidentielle"[..], b"45000", b"salaire"] {
            assert!(
                !bytes.windows(secret.len()).any(|w| w == secret),
                "{name} contient du clair"
            );
        }
        assert!(!name.contains("Confidentielle"));
    }
}

#[tokio::test]
async fn une_cle_incorrecte_n_applique_rien() {
    let space = Space::new();
    let a = space.computer();
    a.add_company("c1", "Acme");
    a.sync_ok().await;

    let other = Space::new();
    let intruder = Computer::detached(&space);
    let mut session = intruder.session.clone();
    session.cipher = Arc::new(SpaceCipher::new(SPACE, &other.key).unwrap());
    intruder.attach(false);
    let report = intruder.engine.cycle(&session).await;
    assert!(matches!(report.failure, Some(SyncFailure::Integrity(_))));
    assert_eq!(intruder.count("SELECT count(*) FROM companies"), 0);
    assert_eq!(intruder.count("SELECT count(*) FROM sync_cursors"), 0);
}

#[tokio::test]
async fn un_lot_altere_est_refuse_sans_avancer_le_curseur() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    a.add_company("c1", "Acme");
    a.sync_ok().await;
    let name = batch_name(&a.session.installation_id, 1);
    let mut bytes = space.remote.raw(&name).unwrap();
    let middle = bytes.len() / 2;
    bytes[middle] ^= 0xff;
    space.remote.remove(&name);
    space.remote.overwrite(&name, bytes);

    let report = b.sync().await;
    assert!(matches!(report.failure, Some(SyncFailure::Integrity(_))));
    assert_eq!(b.count("SELECT count(*) FROM companies"), 0);
    assert_eq!(b.count("SELECT count(*) FROM sync_cursors"), 0);
}

#[tokio::test]
async fn un_lot_d_une_version_plus_recente_est_signale_incompatible() {
    let space = Space::new();
    let b = space.computer();
    let author = uuid::Uuid::new_v4().to_string();
    let name = batch_name(&author, 1);
    let batch = ChangeBatch {
        format: FORMAT_VERSION + 1,
        space_id: SPACE.into(),
        installation_id: author,
        seq: 1,
        schema_version: 99,
        created_at: TS.into(),
        changes: Vec::new(),
    };
    let sealed = b.session.cipher.encode_json(&name, &batch).unwrap();
    space.remote.overwrite(&name, sealed);
    let report = b.sync().await;
    assert!(
        matches!(report.failure, Some(SyncFailure::Incompatible(_))),
        "{report:?}"
    );
}

#[tokio::test]
async fn un_lot_rejoue_sous_une_autre_identite_est_refuse() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    a.add_company("c1", "Acme");
    a.sync_ok().await;
    let original = batch_name(&a.session.installation_id, 1);
    let bytes = space.remote.raw(&original).unwrap();
    let impostor = batch_name(&uuid::Uuid::new_v4().to_string(), 1);
    space.remote.remove(&original);
    space.remote.overwrite(&impostor, bytes);
    let report = b.sync().await;
    assert!(matches!(report.failure, Some(SyncFailure::Integrity(_))));
    assert_eq!(b.count("SELECT count(*) FROM companies"), 0);
}

#[tokio::test]
async fn un_manifeste_malforme_est_refuse() {
    let space = MemoryStore::default();
    space.overwrite("space.json", b"{pas du json".to_vec());
    assert!(matches!(
        read_manifest(&space).await,
        Err(SyncFailure::Integrity(_))
    ));
    space.remove("space.json");
    assert_eq!(read_manifest(&space).await.unwrap(), None);
}

#[tokio::test]
async fn un_manifeste_existant_n_est_jamais_remplace() {
    let remote = MemoryStore::default();
    let first = create_space(SPACE, "phrase secrète de test", TS).unwrap();
    publish_manifest(&remote, &first.manifest).await.unwrap();
    let second = create_space(SPACE, "autre phrase secrète", TS).unwrap();
    assert!(publish_manifest(&remote, &second.manifest).await.is_err());
    assert_eq!(
        read_manifest(&remote).await.unwrap().unwrap(),
        first.manifest
    );
}
