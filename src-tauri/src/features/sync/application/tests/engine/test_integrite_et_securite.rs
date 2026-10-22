//! Données en clair, objets altérés ou incompatibles.

use super::*;
use crate::features::sync::application::engine::{publish_manifest, read_manifest};
use crate::features::sync::domain::protocol::{
    batch_name, ChangeBatch, SpaceManifest, FORMAT_VERSION, MANIFEST_NAME,
};

#[tokio::test]
async fn les_donnees_synchronisees_sont_lisibles_en_clair() {
    let space = Space::new();
    let a = space.computer();
    a.add_company("c1", "Entreprise Confidentielle");
    a.exec("UPDATE companies SET notes = 'salaire souhaité 45000' WHERE id = 'c1'");
    a.sync_ok().await;
    let batch = space
        .remote
        .names()
        .into_iter()
        .find(|name| name.ends_with(".batch"))
        .unwrap();
    let bytes = space.remote.raw(&batch).unwrap();
    assert!(batch.starts_with("v2/changes/"));
    let decoded: ChangeBatch = decode_object(&bytes, &batch).unwrap();
    let json = serde_json::to_vec(&decoded).unwrap();
    for content in [&b"Confidentielle"[..], b"45000", b"salaire"] {
        assert!(json.windows(content.len()).any(|window| window == content));
    }
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
    let bytes = encode_object(&batch).unwrap();
    space.remote.overwrite(&name, bytes);
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
    space.overwrite(MANIFEST_NAME, b"{pas du json".to_vec());
    assert!(matches!(
        read_manifest(&space).await,
        Err(SyncFailure::Integrity(_))
    ));
    space.remove(MANIFEST_NAME);
    assert_eq!(read_manifest(&space).await.unwrap(), None);
}

#[tokio::test]
async fn un_manifeste_existant_n_est_jamais_remplace() {
    let remote = MemoryStore::default();
    let manifest = SpaceManifest {
        format: FORMAT_VERSION,
        app: "candilog".into(),
        space_id: SPACE.into(),
        created_at: TS.into(),
    };
    publish_manifest(&remote, &manifest).await.unwrap();
    assert!(publish_manifest(&remote, &manifest).await.is_err());
    assert_eq!(read_manifest(&remote).await.unwrap().unwrap(), manifest);
}

#[tokio::test]
async fn les_objets_de_l_ancien_espace_ne_sont_pas_reconnus() {
    let space = MemoryStore::default();
    space.overwrite("space.json", "ancien manifeste chiffré".as_bytes().to_vec());
    assert_eq!(read_manifest(&space).await.unwrap(), None);
    assert!(space.raw("space.json").is_some());
}
