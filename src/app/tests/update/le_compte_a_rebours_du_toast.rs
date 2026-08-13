//! Le toast disparaît 4 secondes après sa pose, jamais avant.

use super::*;
use crate::app::state::{NotificationKind, DURATION_AFFICHAGE_TOAST};
use std::time::{Duration, Instant};

/// Un battement reçu avant l'expiration laisse le toast affiché.
#[test]
fn un_battement_precoce_laisse_le_toast_affiche() {
    let mut app = app_de_test();
    app.notify(NotificationKind::Info, "Chargement terminé.");

    app.notification_shown_at = Some(Instant::now() - Duration::from_millis(500));
    let _ = update(&mut app, Message::NotificationCountdown);

    assert!(
        app.notification.is_some(),
        "un toast de moins de 4 s doit rester affiché"
    );
}

/// Après 4 secondes, le battement efface le toast.
#[test]
fn un_battement_apres_expiration_efface_le_toast() {
    let mut app = app_de_test();
    app.notify(NotificationKind::Success, "Entreprise créée.");

    app.notification_shown_at = Some(Instant::now() - DURATION_AFFICHAGE_TOAST);
    let _ = update(&mut app, Message::NotificationCountdown);

    assert_eq!(app.notification, None, "le toast expiré doit disparaître");
    assert_eq!(
        app.notification_shown_at, None,
        "l'horodatage doit être réinitialisé"
    );
}

/// La fermeture manuelle réinitialise aussi l'horodatage.
#[test]
fn la_fermeture_manuelle_reinitialise_l_horodatage() {
    let mut app = app_de_test();
    app.notify(NotificationKind::Info, "Message.");

    let _ = update(&mut app, Message::ClearNotification);

    assert_eq!(app.notification_shown_at, None);
}
