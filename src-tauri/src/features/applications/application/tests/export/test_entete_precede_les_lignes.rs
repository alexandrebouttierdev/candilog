//! Cas de test isolé.

use super::*;

#[test]
fn test_entete_precede_les_lignes() {
    let csv = vers_csv(&[cand("Développeur Frontend", None)]).unwrap();
    let mut rows = csv.lines();

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
