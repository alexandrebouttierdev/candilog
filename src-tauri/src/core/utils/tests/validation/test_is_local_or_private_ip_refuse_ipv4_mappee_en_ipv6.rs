//! Cas de test isolé.

use super::*;

#[test]
fn test_is_local_or_private_ip_refuse_ipv4_mappee_en_ipv6() {
    let mapped: std::net::IpAddr = "::ffff:127.0.0.1".parse().unwrap();
    assert!(is_local_or_private_ip(mapped));
    let cgnat: std::net::IpAddr = "100.64.0.1".parse().unwrap();
    assert!(is_local_or_private_ip(cgnat));
}
