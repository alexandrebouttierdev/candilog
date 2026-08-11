//! Cas de test isolé.

use super::*;

/// Le select de statut vit dans l'inspecteur : sa réussite recharge les données
/// sans fermer la fiche que l'utilisateur est en train de consulter.
#[test]
fn le_changement_de_statut_conserve_la_fiche() {
    let mut app = app_de_test();
    let id = uuid::Uuid::new_v4();
    app.dialog = Some(Dialog::CandidatureDetail(id));

    envoyer(&mut app, [Message::CandidatureStatusUpdated(Ok(()))]);

    assert_eq!(app.dialog, Some(Dialog::CandidatureDetail(id)));
    assert_eq!(
        app.notification.as_ref().map(|notice| notice.kind),
        Some(crate::app::state::NotificationKind::Success),
    );
}
