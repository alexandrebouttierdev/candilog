//! Validations partagées des données reçues à la frontière IPC.

use crate::shared::error::{AppError, AppResult};

/// Valide une URL utilisateur facultative et limite les protocoles à HTTP(S).
///
/// # Errors
/// Retourne `Validation` si l'URL est mal formée ou utilise un protocole dangereux.
pub fn validate_optional_http_url(value: Option<&str>, field: &str) -> AppResult<()> {
    let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
        return Ok(());
    };
    let url = reqwest::Url::parse(value)
        .map_err(|_| AppError::Validation(format!("{field} doit être une URL valide")))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(AppError::Validation(format!(
            "{field} doit utiliser HTTP ou HTTPS"
        )));
    }
    Ok(())
}

/// Indique si une adresse IP appartient à une zone locale ou non routable.
#[must_use]
pub fn is_local_or_private_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(ip) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
        }
        std::net::IpAddr::V6(ip) => {
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || (ip.segments()[0] & 0xfe00) == 0xfc00
                || (ip.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

#[cfg(test)]
#[path = "tests/validation/mod.rs"]
mod tests;
