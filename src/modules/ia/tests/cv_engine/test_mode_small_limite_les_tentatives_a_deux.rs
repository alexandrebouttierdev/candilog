//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_mode_small_limite_les_tentatives_a_deux() {
    let calls = Arc::new(Mutex::new(0));
    let engine = CvEngine::with_mode(
        Arc::new(CountProvider {
            calls: calls.clone(),
        }),
        AnalysisMode::Small,
    );
    assert!(engine.parse_offer("offre").await.is_err());
    assert_eq!(*calls.lock().unwrap(), 2); // Small = 2 tentatives
}
