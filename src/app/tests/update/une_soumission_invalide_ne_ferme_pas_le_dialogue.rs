//! Cas de test isolé.

use super::*;

/// Une soumission qui ne peut pas aboutir doit laisser le dialogue ouvert et prévenir
/// l'utilisateur, sans jamais présenter l'échec comme un succès.
#[test]
fn une_soumission_invalide_ne_ferme_pas_le_dialogue() {
    let mut app = app_de_test();

    // Aucune entreprise n'existe : la candidature ne peut être rattachée à rien.
    envoyer(
        &mut app,
        [
            Message::OpenDialog(Dialog::Candidature),
            Message::CandidaturePosteChanged("Développeur".into()),
            Message::SubmitCandidature,
        ],
    );

    assert_eq!(
        app.dialog,
        Some(Dialog::Candidature),
        "le dialogue doit rester ouvert pour permettre la correction"
    );
    let notice = app
        .notification
        .as_ref()
        .expect("l'utilisateur doit être averti");
    assert_ne!(
        notice.kind,
        crate::app::state::NotificationKind::Success,
        "un échec ne doit jamais s'afficher comme un succès"
    );
}

#[test]
fn une_date_non_francaise_est_refusee_avant_l_ecriture() {
    let mut app = app_de_test();
    app.dialog = Some(Dialog::Candidature);
    app.candidature_form.entreprise_id = Some(uuid::Uuid::new_v4());
    app.candidature_form.poste = "Développeur".into();
    app.candidature_form.date_envoi = "2026/08/11".into();

    envoyer(&mut app, [Message::SubmitCandidature]);

    assert_eq!(app.dialog, Some(Dialog::Candidature));
    let notice = app.notification.as_ref().expect("date invalide signalée");
    assert_eq!(notice.kind, crate::app::state::NotificationKind::Warning);
    assert!(notice.message.contains("JJ-MM-AAAA"));
}
