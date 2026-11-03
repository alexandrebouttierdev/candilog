//! Cas de test isolé.

use super::*;

/// Plages réservées ou non routables que la liste initiale laissait passer : un nom DNS
/// résolu vers l'une d'elles franchissait le refus des adresses locales (`docs/AI.md`).
#[test]
fn test_is_local_or_private_ip_refuse_les_plages_non_routables() {
    for adresse in [
        "0.1.2.3",         // « ce réseau » (0.0.0.0/8)
        "192.0.0.8",       // affectations IETF (192.0.0.0/24)
        "198.18.0.1",      // bancs d'essai (198.18.0.0/15)
        "192.0.2.10",      // documentation TEST-NET-1
        "198.51.100.10",   // documentation TEST-NET-2
        "203.0.113.10",    // documentation TEST-NET-3
        "240.0.0.1",       // réservé (240.0.0.0/4)
        "255.255.255.255", // diffusion
        "64:ff9b::a00:1",  // NAT64 vers 10.0.0.1
        "2002:a00:1::1",   // 6to4 encapsulant 10.0.0.1
        "::10.0.0.1",      // IPv4 compatible
        "2001:db8::1",     // documentation IPv6
    ] {
        assert!(
            is_local_or_private_ip(adresse.parse().unwrap()),
            "{adresse} aurait dû être refusée"
        );
    }
    for adresse in [
        "1.1.1.1",
        "2606:4700:4700::1111",
        "64:ff9b::808:808",
        "2002:808:808::1",
    ] {
        assert!(
            !is_local_or_private_ip(adresse.parse().unwrap()),
            "{adresse} est publique"
        );
    }
}
