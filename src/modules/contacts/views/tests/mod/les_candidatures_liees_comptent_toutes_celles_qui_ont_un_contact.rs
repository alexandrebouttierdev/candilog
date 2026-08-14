//! Cas de test isolé.

use super::*;

#[test]
fn les_candidatures_liees_comptent_toutes_celles_qui_ont_un_contact() {
    let target = Uuid::new_v4();
    let other = Uuid::new_v4();
    let candidates = vec![
        candidature(Some(target)),
        candidature(Some(target)),
        candidature(Some(other)),
        candidature(None),
    ];
    assert_eq!(total_candidatures_liees(&candidates), 3);
    assert_eq!(total_candidatures_liees(&[]), 0);
}
