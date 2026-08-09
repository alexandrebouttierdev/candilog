//! Cas de test isolé.

use super::*;

#[test]
fn test_put_puis_get_retourne_la_valeur() {
    let r = repo();
    r.put(&entry("k1", r#"{"ok":true}"#)).unwrap();
    assert_eq!(r.get("k1").unwrap().as_deref(), Some(r#"{"ok":true}"#));
}
