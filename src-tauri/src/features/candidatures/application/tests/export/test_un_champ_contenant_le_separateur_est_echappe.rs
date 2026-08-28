//! Cas de test isolé.

use super::*;

/// Les notes sont un champ libre : un point-virgule ou un retour à la ligne saisi par
/// l'utilisateur décalerait toutes les colonnes suivantes s'il n'était pas échappé.
#[test]
fn test_un_champ_contenant_le_separateur_est_echappe() {
    let csv = vers_csv(&[cand(
        "Développeur",
        Some("Entretien ; à relancer\nsemaine prochaine"),
    )])
    .unwrap();

    assert!(csv.contains("\"Entretien ; à relancer\nsemaine prochaine\""));

    // Relu par le même dialecte, le fichier doit rendre exactement huit colonnes.
    let mut lecteur = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(csv.as_bytes());
    let ligne = lecteur.records().next().unwrap().unwrap();
    assert_eq!(ligne.len(), 8);
    assert_eq!(&ligne[7], "Entretien ; à relancer\nsemaine prochaine");
}
