//! Validations partagées des données reçues à la frontière IPC.

use std::net::IpAddr;
use std::path::{Component, Path, PathBuf};

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

/// Refuse un chemin vide, un octet nul, ou une traversée `..`.
///
/// Le sélecteur natif fournit un chemin absolu ; les `..` ne servent qu'à un appel IPC forgé.
///
/// # Errors
/// Retourne `Validation` si le chemin n'est pas utilisable tel quel.
pub fn validate_user_file_path(path: impl AsRef<Path>) -> AppResult<PathBuf> {
    let path = path.as_ref();
    if path.as_os_str().is_empty() {
        return Err(AppError::Validation("Chemin de fichier invalide".into()));
    }
    if path.to_str().is_some_and(|value| value.contains('\0'))
        || path.components().any(|c| matches!(c, Component::ParentDir))
    {
        return Err(AppError::Validation("Chemin de fichier invalide".into()));
    }
    Ok(path.to_path_buf())
}

/// Indique si une adresse IP appartient à une zone locale ou non routable.
#[must_use]
pub fn is_local_or_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
                || (ip.octets()[0] == 100 && (ip.octets()[1] & 0b1100_0000) == 64)
        }
        IpAddr::V6(ip) => {
            if let Some(v4) = ip.to_ipv4_mapped() {
                return is_local_or_private_ip(IpAddr::V4(v4));
            }
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
