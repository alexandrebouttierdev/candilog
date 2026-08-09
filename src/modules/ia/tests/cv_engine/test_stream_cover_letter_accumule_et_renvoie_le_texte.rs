//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_stream_cover_letter_accumule_et_renvoie_le_texte() {
    let mut chunks: Vec<String> = Vec::new();
    let req = LetterGenerationRequest {
        tone: Some("formal".into()),
        ..Default::default()
    };
    let full = engine(vec!["Madame, Monsieur, je vous adresse ma candidature."])
        .stream_cover_letter(
            &crate::shared::profile::Profile::default(),
            &req,
            &mut |c: String| chunks.push(c),
        )
        .await
        .unwrap();
    assert!(full.contains("candidature"));
    assert_eq!(chunks.len(), 1); // repli non streamé du provider mock = un seul fragment
}
