//! Cas de test isolé.

use super::*;

#[test]
fn test_gen_options_advanced_sans_contexte_ni_borne() {
    let p = ModeProfile::for_mode(AnalysisMode::Advanced);
    let opts = p.gen_options(Step::History, 12_000);
    assert_eq!(opts.num_ctx, None);
    assert_eq!(opts.num_predict, None);
    assert_eq!(opts.keep_alive, None);
}
