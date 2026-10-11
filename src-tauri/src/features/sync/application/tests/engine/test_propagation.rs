//! Créations, modifications et suppressions se propagent d'un ordinateur à l'autre.

use super::*;

#[tokio::test]
async fn en_mode_local_aucune_modification_n_est_journalisee() {
    let space = Space::new();
    let local = Computer::detached(&space);
    local.add_company("c1", "Acme");
    local.exec("UPDATE companies SET name = 'Acme SA' WHERE id = 'c1'");
    assert_eq!(local.count("SELECT count(*) FROM sync_outbox"), 0);
    assert_eq!(local.engine.store().counts().unwrap().pending, 0);
}

#[tokio::test]
async fn une_creation_arrive_sur_l_autre_ordinateur() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    a.add_company("c1", "Acme");
    a.add_application("app1", "c1", "Développeuse");
    a.add_follow_up("f1", "app1");
    assert_eq!(a.engine.store().counts().unwrap().pending, 3);

    let published = a.sync_ok().await;
    assert_eq!(published.published, 3);
    assert_eq!(a.engine.store().counts().unwrap().pending, 0);
    b.sync_ok().await;

    assert_eq!(
        b.text("SELECT job_title FROM applications WHERE id = 'app1'")
            .as_deref(),
        Some("Développeuse")
    );
    assert_eq!(b.count("SELECT count(*) FROM follow_ups"), 1);
    // Rien de reçu ne repart en écho.
    assert_eq!(b.engine.store().counts().unwrap().pending, 0);
    assert!(b.sync_ok().await.published == 0);
}

#[tokio::test]
async fn une_modification_et_une_suppression_se_propagent() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    a.add_company("c1", "Acme");
    a.add_application("app1", "c1", "Dev");
    a.add_follow_up("f1", "app1");
    a.sync_ok().await;
    b.sync_ok().await;

    b.exec("UPDATE applications SET status = 'ENTRETIEN' WHERE id = 'app1'");
    b.sync_ok().await;
    a.sync_ok().await;
    assert_eq!(
        a.text("SELECT status FROM applications WHERE id = 'app1'")
            .as_deref(),
        Some("ENTRETIEN")
    );

    // La suppression emporte la relance en cascade, des deux côtés.
    a.exec("DELETE FROM applications WHERE id = 'app1'");
    a.sync_ok().await;
    b.sync_ok().await;
    assert_eq!(b.count("SELECT count(*) FROM applications"), 0);
    assert_eq!(b.count("SELECT count(*) FROM follow_ups"), 0);
    assert!(b.conflicts().is_empty());
}

#[tokio::test]
async fn un_lot_recu_deux_fois_ne_cree_aucun_doublon() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    a.add_company("c1", "Acme");
    a.add_application("app1", "c1", "Dev");
    a.sync_ok().await;
    b.sync_ok().await;

    // Curseur effacé : B relit tout l'historique de A.
    b.exec("DELETE FROM sync_cursors");
    b.sync_ok().await;
    b.sync_ok().await;

    assert_eq!(b.count("SELECT count(*) FROM companies"), 1);
    assert_eq!(b.count("SELECT count(*) FROM applications"), 1);
    assert!(b.conflicts().is_empty());
    assert_eq!(b.engine.store().counts().unwrap().pending, 0);
}

#[tokio::test]
async fn une_suppression_ne_reapparait_pas_par_un_lot_ancien_rejoue() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    a.add_company("c1", "Acme");
    a.sync_ok().await;
    b.sync_ok().await;
    a.exec("DELETE FROM companies WHERE id = 'c1'");
    a.sync_ok().await;
    b.sync_ok().await;
    assert_eq!(b.count("SELECT count(*) FROM companies"), 0);

    b.exec("DELETE FROM sync_cursors");
    b.sync_ok().await;
    assert_eq!(
        b.count("SELECT count(*) FROM companies"),
        0,
        "pas de résurrection"
    );
}

#[tokio::test]
async fn un_nouvel_ordinateur_recupere_l_espace_existant() {
    let space = Space::new();
    let a = space.computer();
    a.add_company("c1", "Acme");
    a.add_application("app1", "c1", "Dev");
    a.exec("UPDATE profile SET data = '{\"identity\":{\"first_name\":\"Camille\"}}' WHERE id = 1;
            INSERT OR IGNORE INTO profile (id, data, updated_at) VALUES (1, '{\"identity\":{\"first_name\":\"Camille\"}}', '2026-01-01');");
    a.sync_ok().await;

    // Nouvel ordinateur sans données : il rejoint sans rien publier de vide.
    let fresh = space.computer();
    fresh.sync_ok().await;
    assert_eq!(fresh.count("SELECT count(*) FROM applications"), 1);
    assert!(fresh
        .text("SELECT data FROM profile WHERE id = 1")
        .unwrap()
        .contains("Camille"));
    assert!(
        fresh.conflicts().is_empty(),
        "un profil vide ne fait pas conflit"
    );
}

#[tokio::test]
async fn un_nouvel_ordinateur_part_d_un_instantane() {
    let space = Space::new();
    let a = space.computer();
    a.add_company("c1", "Acme");
    a.sync_ok().await;
    a.exec("UPDATE companies SET city = 'Lyon' WHERE id = 'c1'");
    a.add_company("c2", "Globex");
    a.sync_ok().await;
    let last = a.engine.store().last_published_seq().unwrap();
    a.engine
        .write_snapshot(&a.session, last, Vec::new(), "2000-01-01")
        .await
        .unwrap();
    // Les lots couverts disparaissent (ménage d'une autre installation, rétention écoulée).
    for name in space.remote.names() {
        if name.starts_with("changes/") {
            space.remote.remove(&name);
        }
    }

    let fresh = space.computer();
    fresh.sync_ok().await;
    assert_eq!(fresh.count("SELECT count(*) FROM companies"), 2);
    assert_eq!(
        fresh
            .text("SELECT city FROM companies WHERE id = 'c1'")
            .as_deref(),
        Some("Lyon")
    );

    // Les lots suivants de A s'appliquent après l'instantané.
    a.exec("UPDATE companies SET city = 'Paris' WHERE id = 'c1'");
    a.sync_ok().await;
    fresh.sync_ok().await;
    assert_eq!(
        fresh
            .text("SELECT city FROM companies WHERE id = 'c1'")
            .as_deref(),
        Some("Paris")
    );
}

#[tokio::test]
async fn recuperer_l_espace_remplace_les_donnees_locales() {
    let space = Space::new();
    let a = space.computer();
    a.add_company("c1", "Acme");
    a.sync_ok().await;

    let b = Computer::detached(&space);
    b.add_company("local", "Données locales remplacées");
    let config = b.config();
    b.engine
        .replace_local_with_space(&b.session, &config)
        .await
        .unwrap();
    assert_eq!(
        b.count("SELECT count(*) FROM companies WHERE id = 'local'"),
        0
    );
    assert_eq!(b.count("SELECT count(*) FROM companies WHERE id = 'c1'"), 1);
    assert_eq!(b.engine.store().counts().unwrap().pending, 0);
    // Rattaché : ses modifications suivantes partent.
    b.exec("UPDATE companies SET name = 'Acme SA' WHERE id = 'c1'");
    b.sync_ok().await;
    a.sync_ok().await;
    assert_eq!(
        a.text("SELECT name FROM companies WHERE id = 'c1'")
            .as_deref(),
        Some("Acme SA")
    );
}

#[tokio::test]
async fn fusionner_garde_les_donnees_des_deux_ordinateurs() {
    let space = Space::new();
    let a = space.computer();
    a.add_company("c1", "Acme");
    a.sync_ok().await;

    let b = Computer::detached(&space);
    b.add_company("c2", "Globex");
    b.attach(true);
    b.sync_ok().await;
    a.sync_ok().await;
    for computer in [&a, &b] {
        assert_eq!(computer.count("SELECT count(*) FROM companies"), 2);
    }
}
