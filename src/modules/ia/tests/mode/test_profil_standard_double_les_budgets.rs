//! Cas de test isolé.

use super::*;

#[test]
fn test_profil_standard_double_les_budgets() {
    let p = ModeProfile::for_mode(AnalysisMode::Standard);
    assert_eq!(p.max_attempts, 3);
    assert_eq!(p.output_tokens(Step::Identity), Some(768));
}
