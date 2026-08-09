//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_analyser_entretien_json_valide_extrait_les_champs() {
    let e = engine(vec![
        r#"{"resume":"Bon échange","points_forts":["clarté"],"points_faibles":["trop long"],"suggestions":["citer des chiffres"]}"#,
    ]);
    let a = e
        .analyser_entretien("Compte rendu de l'entretien")
        .await
        .unwrap();
    assert_eq!(a.resume, "Bon échange");
    assert_eq!(a.points_forts, vec!["clarté".to_string()]);
    assert_eq!(a.suggestions, vec!["citer des chiffres".to_string()]);
}
