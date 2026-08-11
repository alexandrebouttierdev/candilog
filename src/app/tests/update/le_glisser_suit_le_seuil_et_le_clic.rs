//! Cas de test isolé.

use super::*;

/// Un appui puis un lâcher sans déplacement nettoie l'état et reste un clic.
#[test]
fn un_appui_pose_puis_lacher_nettoie_l_etat() {
    let mut app = app_de_test();
    let id = uuid::Uuid::new_v4();

    envoyer(&mut app, [Message::CandidatePressed(id)]);
    assert_eq!(app.press_candidate, Some(id));
    assert_eq!(app.dragging_candidate, None);

    envoyer(&mut app, [Message::CandidateReleased]);
    assert_eq!(app.press_candidate, None);
    assert_eq!(app.dragging_candidate, None);
}

/// Un déplacement inférieur au seuil reste un appui, pas un glisser.
#[test]
fn un_deplacement_sous_le_seuil_reste_un_appui() {
    let mut app = app_de_test();
    let id = uuid::Uuid::new_v4();

    envoyer(
        &mut app,
        [
            Message::CandidatePressed(id),
            Message::CandidateMoved(iced::Point::new(10.0, 10.0)),
            Message::CandidateMoved(iced::Point::new(12.0, 12.0)),
        ],
    );
    assert_eq!(app.dragging_candidate, None);
    assert_eq!(app.press_candidate, Some(id));
}

/// Passé le seuil, l'appui devient un glisser et l'appui est oublié.
#[test]
fn un_deplacement_au_dela_du_seuil_demarre_le_glisser() {
    let mut app = app_de_test();
    let id = uuid::Uuid::new_v4();

    envoyer(
        &mut app,
        [
            Message::CandidatePressed(id),
            Message::CandidateMoved(iced::Point::new(10.0, 10.0)),
            Message::CandidateMoved(iced::Point::new(16.0, 10.0)),
        ],
    );
    assert_eq!(app.dragging_candidate, Some(id));
    assert_eq!(app.press_candidate, None);
}

/// Un lâcher hors colonne annule le glisser et vide toute la machine d'état.
#[test]
fn un_lacher_hors_colonne_annule_le_glisser() {
    let mut app = app_de_test();
    let id = uuid::Uuid::new_v4();

    envoyer(
        &mut app,
        [
            Message::CandidatePressed(id),
            Message::CandidateMoved(iced::Point::new(10.0, 10.0)),
            Message::CandidateMoved(iced::Point::new(16.0, 10.0)),
            Message::CandidateDragCancelled,
        ],
    );
    assert_eq!(app.dragging_candidate, None);
    assert_eq!(app.press_candidate, None);
    assert_eq!(app.drag_target_status, None);
}
