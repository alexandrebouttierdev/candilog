//! Cas de test isolé.

use super::*;

/// Excel en locale française attend le point-virgule : un fichier séparé par des virgules
/// s'y ouvre en une seule colonne, ce qui rend l'export inutilisable pour sa fin première,
/// la relecture par l'utilisateur.
#[test]
fn test_le_separateur_est_le_point_virgule() {
    let csv = vers_csv(&[cand("Développeur", None)]).unwrap();
    assert!(csv.contains("Développeur;Nova Digital"));
    assert!(!csv.contains("Développeur,Nova Digital"));
}
