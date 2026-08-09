//! Cas de test isolé.

use super::*;

#[test]
fn test_gen_options_grand_fragment_monte_de_palier() {
    let p = ModeProfile::for_mode(AnalysisMode::Small);
    // ~12000 caractères ⇒ ~4000 tokens d'entrée + marge ⇒ palier 8192 (au-delà de 2048).
    let opts = p.gen_options(Step::History, 12_000);
    assert_eq!(opts.num_ctx, Some(8192));
}
