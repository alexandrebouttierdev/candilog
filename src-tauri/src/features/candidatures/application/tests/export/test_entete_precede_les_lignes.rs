//! Cas de test isolé.

use super::*;

#[test]
fn test_entete_precede_les_lignes() {
    let csv = vers_csv(&[cand("Développeur Frontend", None)]).unwrap();
    let mut lignes = csv.lines();

    assert_eq!(
        lignes.next(),
        Some("poste;entreprise;ville;contrat;statut;date_envoi;lien_offre;notes")
    );
    assert!(lignes
        .next()
        .unwrap()
        .starts_with("Développeur Frontend;Nova Digital;Rennes"));
}
