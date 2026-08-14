//! Cas de test isolé.

use super::*;
use crate::modules::contacts::components::contact_card;

#[test]
fn la_carte_de_contact_s_instancie_avec_et_sans_coordonnees() {
    use iced::Element;

    let bare = contact(None, None);
    let _: Element<'_, ()> = contact_card(&bare, ());
    let complete = contact(Some("DRH"), Some("a@b.fr"));
    let _: Element<'_, ()> = contact_card(&complete, ());
}
