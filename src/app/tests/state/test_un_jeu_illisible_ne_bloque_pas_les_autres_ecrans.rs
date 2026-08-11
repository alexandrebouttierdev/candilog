//! Cas de test isolé.

use super::*;

/// Observé à l'exécution : une ligne de `profil` dont le JSON ne correspondait pas au schéma
/// Rust (un champ manquant) suffisait à afficher **tous** les écrans — Candidatures,
/// Entreprises, Réseau, pourtant parfaitement lisibles en base — en squelette de chargement
/// permanent, avec pour seule indication un toast en bas à droite.
///
/// `reload()` chargeait les onze jeux dans une closure unique où chaque appel propageait par
/// `?` : le moindre échec abandonnait les dix autres et laissait `initialized` à faux, ce qui
/// fige la totalité de l'interface. L'écran Paramètres étant lui aussi un squelette,
/// l'utilisateur n'avait plus aucun moyen de restaurer un backup — et un redémarrage n'y
/// changeait rien.
#[test]
fn test_un_jeu_illisible_ne_bloque_pas_les_autres_ecrans() {
    let backend = BackendState::new().unwrap();
    {
        let connection = backend.sqlite.get().unwrap();
        connection
            .execute_batch(
                "INSERT INTO entreprises (id, nom, created_at, updated_at)
                    VALUES ('11111111-1111-4111-8111-111111111111', 'Acme',
                            '2026-01-01', '2026-01-01');
                 INSERT INTO candidatures
                    (id, entreprise_id, poste, type_contrat, statut, date_envoi,
                     created_at, updated_at)
                    VALUES ('22222222-2222-4222-8222-222222222222',
                            '11111111-1111-4111-8111-111111111111', 'Dev', 'CDI',
                            'EN_ATTENTE', '2026-01-01', '2026-01-01', '2026-01-01');
                 INSERT INTO profil (id, data, updated_at)
                    VALUES (1, '{\"personal\": 42}', '2026-01-01');",
            )
            .unwrap();
    }

    let app = App::with_backend(
        AppPaths::in_directory(std::path::PathBuf::from("/inexistant")),
        backend,
    );

    assert!(
        app.initialized,
        "l'application doit rester utilisable malgré un jeu illisible"
    );
    assert_eq!(
        app.data.candidatures.len(),
        1,
        "les candidatures, parfaitement lisibles, doivent s'afficher"
    );
    assert_eq!(app.data.entreprises.len(), 1);

    let notice = app
        .notification
        .as_ref()
        .expect("l'utilisateur doit être averti de ce qui n'a pas pu être lu");
    assert_eq!(
        notice.kind,
        NotificationKind::Warning,
        "un chargement partiel n'est ni un succès ni une panne totale"
    );
    assert!(
        notice.message.contains("profil"),
        "le message doit nommer le jeu illisible : {}",
        notice.message
    );
}
