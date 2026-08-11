//! Cas de test isolé.

use super::*;

/// Un appui qui ne bouge pas assez reste un clic ; passé le seuil, c'est un glisser.
#[test]
fn un_deplacement_de_5_px_est_un_glisser() {
    let origine = iced::Point::new(10.0, 10.0);
    assert!(!depasse_le_seuil(origine, iced::Point::new(12.0, 12.0)));
    assert!(depasse_le_seuil(origine, iced::Point::new(15.1, 10.0)));
    assert!(depasse_le_seuil(origine, iced::Point::new(10.0, 17.0)));
}
