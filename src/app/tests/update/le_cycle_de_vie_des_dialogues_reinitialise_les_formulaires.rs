//! Cas de test isolé.

use super::*;

/// Ouvrir un dialogue doit repartir d'un formulaire vierge, et le fermer doit oublier l'entité
/// en cours d'édition — sans quoi une création hériterait des valeurs de la modification
/// précédente.
#[test]
fn le_cycle_de_vie_des_dialogues_reinitialise_les_formulaires() {
    let mut app = app_de_test();

    envoyer(
        &mut app,
        [
            Message::OpenDialog(Dialog::Entreprise),
            Message::EntrepriseNomChanged("Acme".into()),
        ],
    );
    assert_eq!(app.dialog, Some(Dialog::Entreprise));
    assert_eq!(app.entreprise_form.nom, "Acme");

    envoyer(&mut app, [Message::CloseDialog]);
    assert_eq!(app.dialog, None);
    assert_eq!(app.editing_id, None);

    // Réouverture : le formulaire est vierge, pas rempli du précédent.
    envoyer(&mut app, [Message::OpenDialog(Dialog::Entreprise)]);
    assert!(
        app.entreprise_form.nom.is_empty(),
        "le formulaire doit repartir vierge"
    );
}

/// Échap ne ferme que ce qui est ouvert. Intercepté globalement et sans condition, il
/// désélectionnait le contact affiché dans l'inspecteur du Réseau — et lui seul, ni la
/// candidature, ni l'entreprise, ni le CV sélectionnés — alors même qu'aucun dialogue n'était
/// ouvert.
#[test]
fn echap_ne_ferme_que_la_couche_ouverte() {
    let mut app = app_de_test();
    let contact = uuid::Uuid::new_v4();

    envoyer(&mut app, [Message::SelectContact(Some(contact))]);

    // Un dialogue ouvert : Échap le ferme, sans toucher à la sélection.
    envoyer(
        &mut app,
        [
            Message::OpenDialog(Dialog::Candidature),
            Message::DismissTopLayer,
        ],
    );
    assert_eq!(app.dialog, None);
    assert_eq!(
        app.selected_contact,
        Some(contact),
        "fermer un dialogue ne doit pas désélectionner le contact"
    );

    // Plus de dialogue : Échap ferme alors la fiche contact.
    envoyer(&mut app, [Message::DismissTopLayer]);
    assert_eq!(app.selected_contact, None);
}
