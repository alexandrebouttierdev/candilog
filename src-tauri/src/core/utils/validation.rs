//! Validations partagées des données reçues à la frontière IPC.

use std::net::IpAddr;

use crate::core::errors::{AppError, AppResult};

/// Valide une URL utilisateur facultative et limite les protocoles à HTTP(S).
///
/// # Errors
/// Retourne `Validation` si l'URL est mal formée ou utilise un protocole dangereux.
pub fn validate_optional_http_url(value: Option<&str>, field: &str) -> AppResult<()> {
    let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
        return Ok(());
    };
    let url = url::Url::parse(value)
        .map_err(|_| AppError::Validation(format!("{field} doit être une URL valide")))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(AppError::Validation(format!(
            "{field} doit utiliser HTTP ou HTTPS"
        )));
    }
    Ok(())
}

/// Indique si une adresse e-mail a une forme utilisable (`local@domaine.tld`).
#[must_use]
pub fn is_valid_email(email: &str) -> bool {
    email.split_once('@').is_some_and(|(local, domain)| {
        !local.is_empty()
            && domain.contains('.')
            && !domain.starts_with('.')
            && !domain.ends_with('.')
    })
}

/// Valide une adresse e-mail facultative.
///
/// # Errors
/// Retourne `Validation` si l'adresse est renseignée mais mal formée.
pub fn validate_optional_email(value: Option<&str>, field: &str) -> AppResult<()> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    if is_valid_email(value) {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "{field} doit être une adresse e-mail valide"
        )))
    }
}

/// Indique si une adresse IP appartient à une zone locale, réservée ou non routable.
///
/// Les formes IPv6 qui **transportent** une adresse IPv4 (mappée, compatible, NAT64, 6to4)
/// sont jugées sur cette adresse : sans quoi `64:ff9b::a00:1` atteindrait `10.0.0.1` en
/// franchissant le refus.
#[must_use]
pub fn is_local_or_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, c, _] = ip.octets();
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
                || ip.is_broadcast()
                || ip.is_documentation()
                // 0.0.0.0/8 : « ce réseau », jamais une destination.
                || a == 0
                // 100.64.0.0/10 : CGNAT.
                || (a == 100 && (b & 0b1100_0000) == 64)
                // 192.0.0.0/24 : affectations IETF.
                || (a == 192 && b == 0 && c == 0)
                // 198.18.0.0/15 : bancs d'essai réseau.
                || (a == 198 && (b & 0xfe) == 18)
                // 240.0.0.0/4 : réservé.
                || a >= 240
        }
        IpAddr::V6(ip) => {
            if let Some(v4) = ipv4_transportee(ip) {
                return is_local_or_private_ip(IpAddr::V4(v4));
            }
            let segments = ip.segments();
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || (segments[0] & 0xfe00) == 0xfc00
                || (segments[0] & 0xffc0) == 0xfe80
                // 2001:db8::/32 : documentation.
                || (segments[0] == 0x2001 && segments[1] == 0x0db8)
        }
    }
}

/// Adresse IPv4 portée par une adresse IPv6 de transition, s'il y en a une.
fn ipv4_transportee(ip: std::net::Ipv6Addr) -> Option<std::net::Ipv4Addr> {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return Some(v4);
    }
    let s = ip.segments();
    let depuis = |haut: u16, bas: u16| {
        let [a, b] = haut.to_be_bytes();
        let [c, d] = bas.to_be_bytes();
        std::net::Ipv4Addr::new(a, b, c, d)
    };
    // `::a.b.c.d` (compatible, obsolète) — hors `::` et `::1`, traités comme IPv6.
    if s[..6] == [0; 6] && !(ip.is_unspecified() || ip.is_loopback()) {
        return Some(depuis(s[6], s[7]));
    }
    // 64:ff9b::/96 : NAT64.
    if s[..6] == [0x64, 0xff9b, 0, 0, 0, 0] {
        return Some(depuis(s[6], s[7]));
    }
    // 2002::/16 : 6to4, l'IPv4 suit le préfixe.
    if s[0] == 0x2002 {
        return Some(depuis(s[1], s[2]));
    }
    None
}

#[cfg(test)]
#[path = "tests/validation/mod.rs"]
mod tests;
