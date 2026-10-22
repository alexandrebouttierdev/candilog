//! CV, lettres et photo du profil : contenus et fichiers joints.

use super::*;

#[tokio::test]
async fn les_cv_et_les_lettres_se_synchronisent() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    a.exec(&format!(
        "INSERT INTO resume_versions (id, name, content, created_at, document_id, version_number, is_current)
         VALUES ('r1', 'CV produit', '{{\"schema_version\":1}}', '{TS}', 'r1', 1, 1);
         INSERT INTO cover_letters (id, name, content, created_at, document_id, version_number, is_current)
         VALUES ('l1', 'Lettre Acme', '<p>Madame, Monsieur</p>', '{TS}', 'l1', 1, 1);"
    ));
    a.sync_ok().await;
    b.sync_ok().await;
    assert_eq!(
        b.text("SELECT content FROM cover_letters WHERE id = 'l1'")
            .as_deref(),
        Some("<p>Madame, Monsieur</p>")
    );
    assert_eq!(
        b.text("SELECT name FROM resume_versions WHERE id = 'r1'")
            .as_deref(),
        Some("CV produit")
    );
}

fn set_photo(computer: &Computer, name: &str, content: &[u8]) {
    std::fs::write(computer.photos.path().join(name), content).unwrap();
    computer.exec(&format!(
        "INSERT INTO profile (id, data, updated_at) VALUES (1, '{{\"photo\":\"{name}\"}}', '{TS}')
         ON CONFLICT(id) DO UPDATE SET data = excluded.data"
    ));
}

#[tokio::test]
async fn la_photo_du_profil_voyage_en_clair_et_est_verifiee() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    let name = format!("photo-{}.png", uuid::Uuid::new_v4());
    set_photo(&a, &name, b"\x89PNG photo d'identite");
    a.sync_ok().await;

    let blobs: Vec<_> = space
        .remote
        .names()
        .into_iter()
        .filter(|n| n.starts_with("v2/blobs/"))
        .collect();
    assert_eq!(blobs.len(), 1);
    let stored = space.remote.raw(&blobs[0]).unwrap();
    assert!(stored.windows(5).any(|w| w == b"photo"), "photo en clair");

    b.sync_ok().await;
    assert_eq!(
        std::fs::read(b.photos.path().join(&name)).unwrap(),
        b"\x89PNG photo d'identite"
    );
}

#[tokio::test]
async fn une_photo_absente_de_l_espace_n_empeche_pas_le_profil() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    let name = format!("photo-{}.png", uuid::Uuid::new_v4());
    set_photo(&a, &name, b"image");
    a.sync_ok().await;
    for blob in space
        .remote
        .names()
        .into_iter()
        .filter(|n| n.starts_with("v2/blobs/"))
    {
        space.remote.remove(&blob);
    }
    b.sync_ok().await;
    assert!(b.text("SELECT data FROM profile").unwrap().contains(&name));
    assert!(
        !b.photos.path().join(&name).exists(),
        "fichier manquant signalé, pas inventé"
    );
    assert_eq!(
        b.engine.store().profile_photo().unwrap().as_deref(),
        Some(name.as_str())
    );
}

#[tokio::test]
async fn une_photo_alteree_est_refusee() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    let name = format!("photo-{}.png", uuid::Uuid::new_v4());
    set_photo(&a, &name, b"image");
    a.sync_ok().await;
    let blob = space
        .remote
        .names()
        .into_iter()
        .find(|n| n.starts_with("v2/blobs/"))
        .unwrap();
    // Un contenu différent sous le même nom ne passe plus la vérification d'empreinte.
    let forged = b"autre image".to_vec();
    space.remote.remove(&blob);
    space.remote.overwrite(&blob, forged);
    b.sync_ok().await;
    assert!(!b.photos.path().join(&name).exists());
}

#[tokio::test]
async fn une_nouvelle_photo_remplace_l_ancienne_sur_l_autre_ordinateur() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    let first = format!("photo-{}.png", uuid::Uuid::new_v4());
    set_photo(&a, &first, b"premiere");
    a.sync_ok().await;
    b.sync_ok().await;
    let second = format!("photo-{}.png", uuid::Uuid::new_v4());
    set_photo(&a, &second, b"seconde");
    a.sync_ok().await;
    b.sync_ok().await;
    assert!(b.photos.path().join(&second).exists());
    assert!(
        !b.photos.path().join(&first).exists(),
        "l'ancienne photo est retirée"
    );
}
