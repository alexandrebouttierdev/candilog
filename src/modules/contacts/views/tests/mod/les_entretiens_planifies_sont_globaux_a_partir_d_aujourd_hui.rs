//! Cas de test isolé.

use super::*;

#[test]
fn les_entretiens_planifies_sont_globaux_a_partir_d_aujourd_hui() {
    let interviews = vec![
        entretien(Some(Uuid::new_v4()), "2026-08-12T09:00:00"),
        entretien(Some(Uuid::new_v4()), "2026-08-08T09:00:00"),
        entretien(None, "2026-08-20T09:00:00"),
    ];
    assert_eq!(entretiens_planifies(&interviews, "2026-08-10"), 2);
    assert_eq!(entretiens_planifies(&interviews, "2026-08-13"), 1);
    assert_eq!(entretiens_planifies(&[], "2026-08-10"), 0);
}
