//! Ouverture explicite de ressources externes depuis une action utilisateur.

/// Ouvre une URL HTTPS dans le navigateur du système.
///
/// # Errors
/// Retourne une erreur lisible si l'URL n'est pas HTTPS, si le lanceur système
/// est absent ou si celui-ci refuse l'ouverture.
pub fn open_https(url: &str) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("Seuls les liens HTTPS peuvent être ouverts.".to_owned());
    }

    #[cfg(target_os = "linux")]
    let status = std::process::Command::new("xdg-open").arg(url).status();

    #[cfg(target_os = "macos")]
    let status = std::process::Command::new("open").arg(url).status();

    #[cfg(target_os = "windows")]
    let status = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .status();

    status
        .map_err(|error| format!("Impossible d'ouvrir le navigateur : {error}"))?
        .success()
        .then_some(())
        .ok_or_else(|| "Le navigateur a refusé l'ouverture du lien.".to_owned())
}

#[cfg(test)]
mod tests {
    #[test]
    fn un_lien_non_securise_est_refuse_avant_tout_lancement() {
        assert!(super::open_https("http://example.com").is_err());
    }
}
