//! Cas de test isolé.

use super::*;

/// Un premier lancement ne doit signaler aucun incident : sans marqueur, il n'y a pas de
/// session précédente à incriminer.
#[test]
fn test_le_premier_lancement_ne_signale_aucun_incident() {
    let directory = tempfile::tempdir().unwrap();
    let marqueur = directory.path().join("candilog.session");

    assert!(!ouvrir_session(&marqueur));
    assert!(
        marqueur.exists(),
        "la session ouverte doit laisser un marqueur"
    );
}

/// Une session fermée proprement retire son marqueur : le lancement suivant est serein.
#[test]
fn test_une_session_fermee_proprement_ne_signale_rien_au_lancement_suivant() {
    let directory = tempfile::tempdir().unwrap();
    let marqueur = directory.path().join("candilog.session");

    ouvrir_session(&marqueur);
    fermer_session(&marqueur);
    assert!(!marqueur.exists(), "la fermeture propre efface le marqueur");

    assert!(!ouvrir_session(&marqueur));
}

/// Un marqueur survivant est la seule trace d'un arrêt que le processus n'a pas pu
/// journaliser : c'est exactement le cas d'un `SIGKILL` de l'OOM killer, où aucune ligne
/// n'est écrite entre la dernière inférence et la mort.
#[test]
fn test_un_marqueur_survivant_revele_un_arret_brutal() {
    let directory = tempfile::tempdir().unwrap();
    let marqueur = directory.path().join("candilog.session");

    ouvrir_session(&marqueur);
    // Aucun appel à `fermer_session` : le processus a été tué.

    assert!(
        ouvrir_session(&marqueur),
        "le marqueur laissé derrière doit révéler l'arrêt brutal"
    );
    assert!(
        marqueur.exists(),
        "la nouvelle session réarme le marqueur pour la suivante"
    );
}

/// Fermer une session sans marqueur ne doit pas échouer : le dossier de données peut avoir
/// été nettoyé pendant l'exécution.
#[test]
fn test_fermer_une_session_sans_marqueur_est_sans_effet() {
    let directory = tempfile::tempdir().unwrap();
    fermer_session(&directory.path().join("candilog.session"));
}
