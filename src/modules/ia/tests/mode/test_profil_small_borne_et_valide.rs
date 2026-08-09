//! Cas de test isolé.

use super::*;

#[test]
fn test_profil_small_borne_et_valide() {
    let p = ModeProfile::for_mode(AnalysisMode::Small);
    assert!(p.size_context && p.cap_output && p.grounding);
    assert!(!p.parallel_sections);
    assert_eq!(p.max_attempts, 2);
    assert_eq!(p.output_tokens(Step::Identity), Some(384));
}
