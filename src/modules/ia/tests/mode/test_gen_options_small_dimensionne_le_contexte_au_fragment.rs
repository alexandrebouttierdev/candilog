//! Cas de test isolé.

use super::*;

#[test]
fn test_gen_options_small_dimensionne_le_contexte_au_fragment() {
    let p = ModeProfile::for_mode(AnalysisMode::Small);
    // Petit fragment → petit contexte (palier 2048), sortie bornée, keep_alive présent.
    let opts = p.gen_options(Step::Identity, 300);
    assert_eq!(opts.num_ctx, Some(2048));
    assert_eq!(opts.num_predict, Some(384));
    assert_eq!(opts.keep_alive, Some("10m"));
}
