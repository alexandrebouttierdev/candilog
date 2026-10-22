//! Modifications concurrentes : fusion quand c'est sûr, conflit présenté sinon, jamais de
//! perte silencieuse.

use super::*;
use crate::features::sync::domain::merge::ConflictKind;
use crate::features::sync::domain::model::ConflictChoice;

async fn shared_company() -> (Space, Computer, Computer) {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    a.add_company("c1", "Acme");
    a.sync_ok().await;
    b.sync_ok().await;
    (space, a, b)
}

#[tokio::test]
async fn deux_enregistrements_distincts_modifies_en_parallele_se_rejoignent() {
    let (_space, a, b) = shared_company().await;
    b.add_company("c2", "Globex");
    b.sync_ok().await;
    a.sync_ok().await;
    a.exec("UPDATE companies SET city = 'Lyon' WHERE id = 'c1'");
    b.exec("UPDATE companies SET city = 'Paris' WHERE id = 'c2'");
    a.sync_ok().await;
    b.sync_ok().await;
    a.sync_ok().await;
    for computer in [&a, &b] {
        assert_eq!(
            computer
                .text("SELECT city FROM companies WHERE id = 'c1'")
                .as_deref(),
            Some("Lyon")
        );
        assert_eq!(
            computer
                .text("SELECT city FROM companies WHERE id = 'c2'")
                .as_deref(),
            Some("Paris")
        );
        assert!(computer.conflicts().is_empty());
    }
}

#[tokio::test]
async fn deux_champs_d_une_meme_fiche_fusionnent() {
    let (_space, a, b) = shared_company().await;
    a.exec("UPDATE companies SET city = 'Lyon', updated_at = '2026-03-01T00:00:00+00:00' WHERE id = 'c1'");
    b.exec("UPDATE companies SET website = 'https://acme.test', updated_at = '2026-03-02T00:00:00+00:00' WHERE id = 'c1'");
    a.sync_ok().await;
    b.sync_ok().await;
    a.sync_ok().await;
    for computer in [&a, &b] {
        assert_eq!(
            computer.text("SELECT city FROM companies").as_deref(),
            Some("Lyon")
        );
        assert_eq!(
            computer.text("SELECT website FROM companies").as_deref(),
            Some("https://acme.test")
        );
        assert_eq!(
            computer.text("SELECT updated_at FROM companies").as_deref(),
            Some("2026-03-02T00:00:00+00:00")
        );
        assert!(computer.conflicts().is_empty());
    }
}

#[tokio::test]
async fn le_meme_champ_modifie_des_deux_cotes_est_presente_puis_resolu_partout() {
    let (_space, a, b) = shared_company().await;
    a.exec("UPDATE companies SET city = 'Lyon' WHERE id = 'c1'");
    b.exec("UPDATE companies SET city = 'Paris' WHERE id = 'c1'");
    a.sync_ok().await;
    b.sync_ok().await;
    a.sync_ok().await;

    // Chaque côté garde sa valeur et voit le conflit : rien n'est écrasé en silence.
    assert_eq!(
        a.text("SELECT city FROM companies").as_deref(),
        Some("Lyon")
    );
    assert_eq!(
        b.text("SELECT city FROM companies").as_deref(),
        Some("Paris")
    );
    let on_b = b.conflicts();
    assert_eq!(on_b.len(), 1);
    assert_eq!(on_b[0].kind, ConflictKind::Edit);
    assert_eq!(on_b[0].fields[0].remote, "Lyon");
    assert_eq!(a.conflicts().len(), 1);

    // B garde « Paris » : la décision est publiée et A l'adopte.
    b.engine
        .store()
        .resolve_conflict(&on_b[0].id, ConflictChoice::KeepLocal)
        .unwrap();
    b.sync_ok().await;
    a.sync_ok().await;
    assert_eq!(
        a.text("SELECT city FROM companies").as_deref(),
        Some("Paris")
    );
    assert!(
        a.conflicts().is_empty(),
        "le conflit est levé par la décision publiée"
    );
    assert!(b.conflicts().is_empty());
    // Plus rien ne circule : la convergence est stable.
    assert_eq!(a.sync_ok().await.published, 0);
    assert_eq!(b.sync_ok().await.published, 0);
}

#[tokio::test]
async fn prendre_la_version_distante_l_applique_partout() {
    let (_space, a, b) = shared_company().await;
    a.exec("UPDATE companies SET city = 'Lyon' WHERE id = 'c1'");
    b.exec("UPDATE companies SET city = 'Paris' WHERE id = 'c1'");
    a.sync_ok().await;
    b.sync_ok().await;
    let conflict = b.conflicts().remove(0);
    b.engine
        .store()
        .resolve_conflict(&conflict.id, ConflictChoice::TakeRemote)
        .unwrap();
    assert_eq!(
        b.text("SELECT city FROM companies").as_deref(),
        Some("Lyon")
    );
    b.sync_ok().await;
    a.sync_ok().await;
    assert_eq!(
        a.text("SELECT city FROM companies").as_deref(),
        Some("Lyon")
    );
    assert!(a.conflicts().is_empty());
}

#[tokio::test]
async fn supprime_ici_et_modifie_ailleurs_est_un_conflit_des_deux_cotes() {
    let (_space, a, b) = shared_company().await;
    a.exec("DELETE FROM companies WHERE id = 'c1'");
    b.exec("UPDATE companies SET city = 'Paris' WHERE id = 'c1'");
    a.sync_ok().await;
    b.sync_ok().await;
    a.sync_ok().await;

    let on_b = b.conflicts();
    assert_eq!(on_b[0].kind, ConflictKind::DeletedRemotely);
    assert_eq!(
        b.count("SELECT count(*) FROM companies"),
        1,
        "la fiche modifiée reste"
    );
    let on_a = a.conflicts();
    assert_eq!(on_a[0].kind, ConflictKind::DeletedLocally);

    // B conserve la fiche : elle réapparaît sur A.
    b.engine
        .store()
        .resolve_conflict(&on_b[0].id, ConflictChoice::KeepLocal)
        .unwrap();
    b.sync_ok().await;
    a.sync_ok().await;
    assert_eq!(
        a.text("SELECT city FROM companies WHERE id = 'c1'")
            .as_deref(),
        Some("Paris")
    );
    assert!(a.conflicts().is_empty());
}

#[tokio::test]
async fn confirmer_la_suppression_l_applique_partout() {
    let (_space, a, b) = shared_company().await;
    a.exec("DELETE FROM companies WHERE id = 'c1'");
    b.exec("UPDATE companies SET city = 'Paris' WHERE id = 'c1'");
    a.sync_ok().await;
    b.sync_ok().await;
    let conflict = b.conflicts().remove(0);
    b.engine
        .store()
        .resolve_conflict(&conflict.id, ConflictChoice::TakeRemote)
        .unwrap();
    b.sync_ok().await;
    a.sync_ok().await;
    for computer in [&a, &b] {
        assert_eq!(computer.count("SELECT count(*) FROM companies"), 0);
    }
    assert!(a.conflicts().is_empty());
}

#[tokio::test]
async fn une_suppression_distante_n_emporte_pas_un_enfant_cree_ici() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    a.add_company("c1", "Acme");
    a.add_application("app1", "c1", "Dev");
    a.sync_ok().await;
    b.sync_ok().await;

    a.exec("DELETE FROM applications WHERE id = 'app1'");
    b.add_follow_up("f-local", "app1");
    a.sync_ok().await;
    b.sync_ok().await;

    assert_eq!(
        b.count("SELECT count(*) FROM follow_ups WHERE id = 'f-local'"),
        1
    );
    assert_eq!(b.conflicts()[0].kind, ConflictKind::DeletedRemotely);
}

#[tokio::test]
async fn un_enfant_cree_ailleurs_restaure_son_parent_supprime_ici() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    a.add_company("c1", "Acme");
    a.sync_ok().await;
    b.sync_ok().await;

    a.exec("DELETE FROM companies WHERE id = 'c1'");
    b.add_application("app1", "c1", "Dev");
    a.sync_ok().await;
    b.sync_ok().await;
    a.sync_ok().await;
    b.sync_ok().await;

    // La candidature créée ailleurs n'est pas perdue : son entreprise est restaurée.
    assert_eq!(a.count("SELECT count(*) FROM applications"), 1);
    assert_eq!(a.count("SELECT count(*) FROM companies"), 1);
    assert_eq!(b.count("SELECT count(*) FROM companies"), 1);
}

#[tokio::test]
async fn deux_candidatures_creees_hors_ligne_gardent_des_references_distinctes() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    a.add_company("c1", "Acme");
    a.sync_ok().await;
    b.sync_ok().await;

    a.add_application("aaaa", "c1", "Sur A");
    b.add_application("bbbb", "c1", "Sur B");
    assert_eq!(a.count("SELECT reference_number FROM applications"), 1);
    assert_eq!(b.count("SELECT reference_number FROM applications"), 1);

    for _ in 0..3 {
        a.sync_ok().await;
        b.sync_ok().await;
    }
    for computer in [&a, &b] {
        assert_eq!(
            computer.count("SELECT count(DISTINCT reference_number) FROM applications"),
            2
        );
        assert_eq!(
            computer.count("SELECT reference_number FROM applications WHERE id = 'aaaa'"),
            1
        );
        assert!(computer.conflicts().is_empty());
    }
    assert_eq!(
        a.count("SELECT reference_number FROM applications WHERE id = 'bbbb'"),
        b.count("SELECT reference_number FROM applications WHERE id = 'bbbb'"),
        "les deux ordinateurs convergent"
    );
}

#[tokio::test]
async fn deux_versions_courantes_concurrentes_convergent_sans_perte() {
    let space = Space::new();
    let a = space.computer();
    let b = space.computer();
    a.exec(&format!(
        "INSERT INTO resume_versions (id, name, content, created_at, document_id, version_number, is_current)
         VALUES ('v1', 'CV', '{{}}', '{TS}', 'v1', 1, 1)"
    ));
    a.sync_ok().await;
    b.sync_ok().await;

    a.exec(&format!(
        "UPDATE resume_versions SET is_current = 0 WHERE id = 'v1';
         INSERT INTO resume_versions (id, name, content, created_at, document_id, version_number, is_current)
         VALUES ('va', 'CV', '{{}}', '{TS}', 'v1', 2, 1)"
    ));
    b.exec(&format!(
        "UPDATE resume_versions SET is_current = 0 WHERE id = 'v1';
         INSERT INTO resume_versions (id, name, content, created_at, document_id, version_number, is_current)
         VALUES ('vb', 'CV', '{{}}', '{TS}', 'v1', 2, 1)"
    ));
    for _ in 0..3 {
        a.sync_ok().await;
        b.sync_ok().await;
    }
    for computer in [&a, &b] {
        assert_eq!(
            computer.count("SELECT count(*) FROM resume_versions"),
            3,
            "aucune version perdue"
        );
        assert_eq!(
            computer.count("SELECT count(*) FROM resume_versions WHERE is_current = 1"),
            1
        );
        assert_eq!(
            computer
                .text("SELECT id FROM resume_versions WHERE is_current = 1")
                .as_deref(),
            Some("vb")
        );
    }
}
