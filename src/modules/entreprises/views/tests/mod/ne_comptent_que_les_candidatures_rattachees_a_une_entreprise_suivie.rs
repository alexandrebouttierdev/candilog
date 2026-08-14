//! Cas de test isolé.

use super::*;

#[test]
fn ne_comptent_que_les_candidatures_rattachees_a_une_entreprise_suivie() {
    let target = Uuid::new_v4();
    let other = Uuid::new_v4();
    let candidates = vec![candidature(target), candidature(target), candidature(other)];
    assert_eq!(total_candidatures(&candidates, &[entreprise(target)]), 2);
    assert_eq!(total_candidatures(&candidates, &[entreprise(other)]), 1);
}
