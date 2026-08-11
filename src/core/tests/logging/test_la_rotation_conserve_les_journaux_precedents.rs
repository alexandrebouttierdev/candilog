//! Cas de test isolé.

use super::*;

/// La rotation au démarrage doit décaler les journaux existants sans jamais écraser le
/// précédent : le journal utile après un incident est celui de la session **d'avant**, pas
/// celui de la session de diagnostic que l'utilisateur vient de lancer.
#[test]
fn test_la_rotation_conserve_les_journaux_precedents() {
    let directory = tempfile::tempdir().unwrap();
    let journal = directory.path().join("candilog.log");

    for session in 1..=3 {
        std::fs::write(&journal, format!("session {session}")).unwrap();
        faire_tourner(&journal);
    }

    // La session 3 vient d'être décalée en .1, la 2 en .2, la 1 en .3.
    for (rang, attendu) in [(1, "session 3"), (2, "session 2"), (3, "session 1")] {
        let contenu = std::fs::read_to_string(journal.with_extension(format!("log.{rang}")))
            .unwrap_or_else(|_| panic!("le journal .{rang} devrait exister"));
        assert_eq!(contenu, attendu);
    }
    assert!(
        !journal.exists(),
        "le journal courant a été décalé, un nouveau sera créé à l'ouverture"
    );
}

/// Faire tourner un journal absent ne doit rien produire ni échouer : c'est le cas du tout
/// premier lancement.
#[test]
fn test_la_rotation_d_un_journal_absent_est_sans_effet() {
    let directory = tempfile::tempdir().unwrap();
    let journal = directory.path().join("candilog.log");
    faire_tourner(&journal);
    assert!(!journal.with_extension("log.1").exists());
}
