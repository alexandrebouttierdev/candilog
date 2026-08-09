//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_parse_offer_toujours_invalide_retourne_erreur() {
    let r = engine(vec!["nope", "encore nope", "toujours pas"])
        .parse_offer("offre")
        .await;
    assert!(matches!(r, Err(AppError::Serialization(_))));
}
