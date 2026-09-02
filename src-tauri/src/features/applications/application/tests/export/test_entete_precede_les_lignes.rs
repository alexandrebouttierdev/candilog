//! Cas de test isolé.

use super::*;

#[test]
fn test_entete_precede_les_lignes() {
    let csv = vers_csv(&[cand("Développeur Frontend", None)]).unwrap();
    // La marque d'ordre d'octets ouvre le fichier pour Excel ; l'en-tête la suit
    // immédiatement (`core::utils::csv_export`).
    let mut rows = csv.trim_start_matches('\u{feff}').lines();

    assert_eq!(
        rows.next(),
        Some(
            "poste;entreprise;type_candidature;contrat;duree_hebdomadaire;heures_par_semaine;\
             domaine_professionnel;type_entreprise;taille_entreprise;ville;adresse;statut;\
             date_envoi;lien_offre;notes"
        )
    );
    assert!(rows
        .next()
        .unwrap()
        .starts_with("Développeur Frontend;Nova Digital;Offre d'emploi;CDI"));
}
