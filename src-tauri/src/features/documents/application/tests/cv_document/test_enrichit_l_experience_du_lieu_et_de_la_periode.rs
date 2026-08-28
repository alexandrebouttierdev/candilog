//! Cas de test isolé.

use super::*;

#[test]
fn enrichit_l_experience_du_lieu_et_de_la_periode() {
    let cv = construire(&profil(), &generation());
    assert_eq!(cv.experiences.len(), 1);
    assert_eq!(cv.experiences[0].meta, "Rennes · Juil. 2019 – Oct. 2025");
    assert_eq!(cv.experiences[0].bullets, vec!["Une description."]);
}
