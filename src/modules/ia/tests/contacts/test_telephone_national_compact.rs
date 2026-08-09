//! Cas de test isolé.

use super::*;

#[test]
fn test_telephone_national_compact() {
    let c = extract_contacts("Tel: 0612345678");
    assert_eq!(c.phone.as_deref(), Some("0612345678"));
}
