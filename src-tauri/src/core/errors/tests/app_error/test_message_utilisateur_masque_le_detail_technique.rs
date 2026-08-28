//! Cas de test isolé.

use super::*;

/// Les erreurs d'infrastructure étaient présentées telles quelles : l'utilisateur lisait
/// « Base de données : timed out waiting for connection: unable to open database file:
/// /home/…/candilog.sqlite » — du texte technique anglais et l'arborescence locale complète,
/// ni compréhensible ni actionnable, et divulguée dans toute capture d'écran de support.
///
/// Le détail technique appartient au journal ; l'écran reçoit une reformulation.
#[test]
fn test_message_utilisateur_masque_le_detail_technique() {
    let brute = "timed out waiting for connection: unable to open database file: \
                 /home/alex/.local/share/com.candilog.desktop/candilog.sqlite";
    let message = AppError::Database(brute.into()).message_utilisateur();

    assert!(
        !message.contains('/'),
        "aucun chemin ne doit apparaître : {message}"
    );
    assert!(
        !message.contains("timed out"),
        "aucun texte technique anglais ne doit apparaître : {message}"
    );
    assert!(
        message.contains("données"),
        "le message doit rester explicite pour l'utilisateur : {message}"
    );

    // La sérialisation souffre du même défaut : « missing field 'first_name' at line 1
    // column 344 » s'affichait à l'écran.
    let serialisation =
        AppError::Serialization("missing field `first_name` at line 1 column 344".into())
            .message_utilisateur();
    assert!(
        !serialisation.contains("missing field"),
        "le détail serde doit rester dans le journal : {serialisation}"
    );
}

/// Les messages écrits par la validation métier sont, eux, déjà destinés à l'utilisateur :
/// les masquer priverait de toute indication sur le champ fautif.
#[test]
fn test_message_utilisateur_conserve_les_messages_de_validation() {
    let message = AppError::Validation("Le poste est obligatoire.".into()).message_utilisateur();
    assert_eq!(message, "Le poste est obligatoire.");
}
