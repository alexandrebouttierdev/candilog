//! Cas de test isolé.

use super::*;

#[test]
fn test_is_local_or_private_ip_distingue_adresses() {
    assert!(is_local_or_private_ip("127.0.0.1".parse().unwrap()));
    assert!(is_local_or_private_ip("192.168.1.2".parse().unwrap()));
    assert!(!is_local_or_private_ip("8.8.8.8".parse().unwrap()));
}
