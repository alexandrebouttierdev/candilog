//! Contrat d'une page A4 unique.

use super::*;

#[test]
fn page_contract_est_a4_et_une_page() {
    assert_eq!(A4.width_mm, 210.0);
    assert_eq!(A4.height_mm, 297.0);
    assert_eq!(A4.width_pt, 595.28);
    assert_eq!(A4.height_pt, 841.89);
}

#[test]
fn bornes_refusent_un_depassement_des_marges() {
    let margins = Margins::uniform(20.0);
    assert!(LayoutBounds {
        max_x: A4.width_pt - 20.0,
        max_y: A4.height_pt - 20.0,
    }
    .fits(A4, margins));
    assert!(!LayoutBounds {
        max_x: A4.width_pt - 19.0,
        max_y: A4.height_pt - 20.0,
    }
    .fits(A4, margins));
}
