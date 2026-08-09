//! Cas de test isolé.

use super::*;

#[tokio::test]
async fn test_mode_standard_autorise_trois_tentatives() {
    let calls = Arc::new(Mutex::new(0));
    let engine = CvEngine::with_mode(
        Arc::new(CountProvider {
            calls: calls.clone(),
        }),
        AnalysisMode::Standard,
    );
    assert!(engine.parse_offer("offre").await.is_err());
    assert_eq!(*calls.lock().unwrap(), 3); // Standard = 3 tentatives
}
