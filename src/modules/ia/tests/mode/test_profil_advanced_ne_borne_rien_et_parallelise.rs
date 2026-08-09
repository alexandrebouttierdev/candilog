//! Cas de test isolé.

use super::*;

#[test]
fn test_profil_advanced_ne_borne_rien_et_parallelise() {
    let p = ModeProfile::for_mode(AnalysisMode::Advanced);
    assert!(!p.size_context && !p.cap_output && !p.grounding);
    assert!(p.parallel_sections);
    assert_eq!(p.output_tokens(Step::History), None);
}
