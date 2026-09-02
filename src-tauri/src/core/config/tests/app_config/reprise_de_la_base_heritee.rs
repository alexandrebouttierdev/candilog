//! Cas de test isolé.

use super::*;

/// Le dossier de données a changé de nom pour s'aligner sur l'identifiant du paquet. Sans
/// reprise, la base d'un poste existant deviendrait invisible et l'application ouvrirait
/// une base vide — l'utilisateur croirait avoir tout perdu.
#[test]
fn la_base_heritee_est_deplacee_vers_le_nouveau_dossier() {
    let racine = tempfile::tempdir().unwrap();
    let ancien = racine.path().join("com.candilog.desktop");
    let nouveau = racine.path().join("fr.candilog.desktop");
    std::fs::create_dir_all(&ancien).unwrap();
    std::fs::write(ancien.join("candilog.sqlite"), b"base").unwrap();
    std::fs::write(ancien.join("candilog.sqlite-wal"), b"wal").unwrap();

    reprendre_base_heritee(&ancien, &nouveau);

    assert_eq!(
        std::fs::read(nouveau.join("candilog.sqlite")).unwrap(),
        b"base"
    );
    assert_eq!(
        std::fs::read(nouveau.join("candilog.sqlite-wal")).unwrap(),
        b"wal",
        "le journal WAL porte des transactions non intégrées : le laisser les perdrait"
    );
    assert!(
        !ancien.join("candilog.sqlite").exists(),
        "déplacement, pas copie"
    );
}

/// Une base déjà présente appartient à l'installation courante : la reprise ne doit jamais
/// l'écraser par une base plus ancienne restée à côté.
#[test]
fn une_base_deja_presente_n_est_jamais_ecrasee() {
    let racine = tempfile::tempdir().unwrap();
    let ancien = racine.path().join("com.candilog.desktop");
    let nouveau = racine.path().join("fr.candilog.desktop");
    std::fs::create_dir_all(&ancien).unwrap();
    std::fs::create_dir_all(&nouveau).unwrap();
    std::fs::write(ancien.join("candilog.sqlite"), b"ancienne").unwrap();
    std::fs::write(nouveau.join("candilog.sqlite"), b"courante").unwrap();

    reprendre_base_heritee(&ancien, &nouveau);

    assert_eq!(
        std::fs::read(nouveau.join("candilog.sqlite")).unwrap(),
        b"courante"
    );
    assert!(
        ancien.join("candilog.sqlite").exists(),
        "l'ancienne reste récupérable"
    );
}

/// Sans rien à reprendre, la reprise ne doit créer aucun dossier : une installation neuve
/// n'a pas à porter la trace d'un emplacement qu'elle n'a jamais utilisé.
#[test]
fn sans_base_heritee_rien_n_est_cree() {
    let racine = tempfile::tempdir().unwrap();
    let ancien = racine.path().join("com.candilog.desktop");
    let nouveau = racine.path().join("fr.candilog.desktop");

    reprendre_base_heritee(&ancien, &nouveau);

    assert!(!nouveau.exists());
}

/// L'identifiant du dossier de données est celui du paquet : si les deux divergent, la
/// désinstallation et le nettoyage manuel visent un emplacement introuvable.
#[test]
fn l_identifiant_est_celui_du_paquet() {
    let manifeste: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../tauri.conf.json")).unwrap();
    assert_eq!(manifeste["identifier"].as_str(), Some(APP_IDENTIFIER));
}
