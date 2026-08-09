//! Cas de test isolé.

use super::*;

#[test]
fn test_origine_score_aller_retour() {
    for o in [OrigineScore::Genere, OrigineScore::Importe] {
        assert_eq!(OrigineScore::depuis_str(o.as_str()), Some(o));
    }
    assert_eq!(OrigineScore::depuis_str("autre"), None);
}
