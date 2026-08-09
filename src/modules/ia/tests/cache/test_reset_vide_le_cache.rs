//! Cas de test isolé.

use super::*;

#[test]
fn test_reset_vide_le_cache() {
    let r = repo();
    r.put(&entry("k1", "v1")).unwrap();
    r.reset().unwrap();
    assert_eq!(r.get("k1").unwrap(), None);
}
