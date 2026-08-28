//! Cas de test isolé.

use super::*;

#[test]
fn test_entete_precede_les_lignes() {
    let csv = vers_csv(&[cand("Développeur Frontend", None)]).unwrap();
    let mut rows = csv.lines();

    assert_eq!(
        rows.next(),
        Some("poste;entreprise;ville;contrat;statut;sent_date;job_url;notes")
    );
    assert!(rows
        .next()
        .unwrap()
        .starts_with("Développeur Frontend;Nova Digital;Rennes"));
}
