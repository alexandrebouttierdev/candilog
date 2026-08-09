//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_analyser_entretien_toujours_invalide_retourne_erreur() {
    let e = engine(vec!["pas du json", "toujours pas", "encore non"]);
    assert!(e.analyser_entretien("cr").await.is_err());
}
