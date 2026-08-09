//! Cas de test isolé.

use super::*;

#[test]
fn test_put_remplace_une_cle_existante() {
    let r = repo();
    r.put(&entry("k1", "v1")).unwrap();
    r.put(&entry("k1", "v2")).unwrap();
    assert_eq!(r.get("k1").unwrap().as_deref(), Some("v2"));
}
