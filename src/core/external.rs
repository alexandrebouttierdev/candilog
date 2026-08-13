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

/// Joue le son de fin d'analyse IA, de façon bloquante.
///
/// À appeler **hors du fil de rendu** (voir [`crate::app::commandes::sonner_fin_analyse`]) :
/// le lanceur audio est attendu (`status`) afin de récolter le processus enfant et de ne pas
/// laisser de zombie derrière chaque génération. Meilleur effort : l'absence ou l'échec d'un
/// lanceur est ignoré, le son étant un confort et non un contrat.
pub fn jouer_son_analyse() {
    #[cfg(target_os = "linux")]
    {
        // `canberra-gtk-play -i complete` respecte le thème sonore du bureau ;
        // `paplay`/`pw-play` couvrent PulseAudio et PipeWire natif en dernier recours.
        let lanceurs: [(&str, &[&str]); 3] = [
            ("canberra-gtk-play", &["-i", "complete"]),
            (
                "paplay",
                &["/usr/share/sounds/freedesktop/stereo/complete.oga"],
            ),
            (
                "pw-play",
                &["/usr/share/sounds/freedesktop/stereo/complete.oga"],
            ),
        ];
        for (commande, args) in lanceurs {
            if std::process::Command::new(commande)
                .args(args)
                .status()
                .is_ok()
            {
                return;
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("afplay")
            .arg("/System/Library/Sounds/Glass.aiff")
            .status();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "(New-Object Media.SoundPlayer 'C:\\Windows\\Media\\notify.wav').PlaySync()",
            ])
            .status();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn un_lien_non_securise_est_refuse_avant_tout_lancement() {
        assert!(super::open_https("http://example.com").is_err());
    }
}
