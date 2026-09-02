//! Cas de test isolé.

use super::*;

/// Le message d'un dossier de données inutilisable est le seul que verra un utilisateur
/// dont l'application ne démarre pas : il doit nommer la vraie cause.
///
/// La variante `Database` répondait « Le fichier de données de Candilog est illisible ou
/// endommagé » à quelqu'un dont le disque est plein — un diagnostic faux, qui envoie
/// chercher une corruption inexistante au lieu de libérer de l'espace.
#[test]
fn un_dossier_de_donnees_inutilisable_ne_parle_pas_de_base_corrompue() {
    // Un fichier régulier là où un dossier est attendu : `create_dir_all` échoue, comme
    // pour un disque plein ou un dossier en lecture seule.
    let racine = tempfile::tempdir().unwrap();
    let occupe = racine.path().join("data");
    std::fs::write(&occupe, b"pas un dossier").unwrap();

    let echec = AppPaths::discover_dans(&occupe).expect_err("le dossier ne peut pas être créé");

    let message = echec.user_message();
    assert!(
        !message.contains("illisible ou endommagé"),
        "diagnostic de base corrompue pour un problème de dossier : {message}"
    );
    assert!(
        message.contains(&occupe.display().to_string()),
        "le message doit nommer le dossier à corriger : {message}"
    );
    assert!(
        message.contains("droits d'écriture") && message.contains("espace disque"),
        "le message doit dire quoi vérifier : {message}"
    );
    assert!(
        !message.contains("os error"),
        "l'erreur système brute est en anglais : elle appartient au journal ({message})"
    );
}
