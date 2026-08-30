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

    // Relu par le même dialecte, le fichier doit rendre exactement ses colonnes.
    let mut lecteur = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(csv.as_bytes());
    let row = lecteur.records().next().unwrap().unwrap();
    assert_eq!(row.len(), COLONNES);
    assert_eq!(
        &row[COLONNES - 1],
        "Entretien ; à relancer\nsemaine prochaine"
    );
}
