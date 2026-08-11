//! Détection du thème clair/sombre du système d'exploitation.
//!
//! `ThemePref::System` n'était qu'un libellé : le choix était traité **exactement comme**
//! « Sombre » (`!matches!(value, Light)`), si bien qu'un utilisateur dont le système est en
//! clair et qui choisissait « Système » obtenait le thème sombre, sans explication. Aucune
//! détection n'existait dans le code.
//!
//! La détection est **asynchrone** parce que le portail XDG l'est : elle passe donc par une
//! `Task`, et son résultat est conservé dans l'état plutôt que redemandé à chaque rendu.

/// Interroge le système. `None` quand il ne se prononce pas ou n'est pas interrogeable — le
/// thème courant est alors conservé, plutôt que d'imposer une bascule arbitraire.
pub async fn detecter() -> Option<bool> {
    let resultat = detecter_plateforme().await;
    match resultat {
        Some(sombre) => tracing::debug!(sombre, "thème système détecté"),
        None => tracing::debug!("thème système non déterminé"),
    }
    resultat
}

/// Linux et BSD : portail XDG `org.freedesktop.appearance` / `color-scheme`.
#[cfg(target_os = "linux")]
async fn detecter_plateforme() -> Option<bool> {
    use ashpd::desktop::settings::{ColorScheme, Settings};

    // Le portail est absent des sessions minimales : son indisponibilité n'est pas une erreur.
    let settings = Settings::new().await.ok()?;
    match settings.color_scheme().await.ok()? {
        ColorScheme::PreferDark => Some(true),
        ColorScheme::PreferLight => Some(false),
        ColorScheme::NoPreference => None,
    }
}

/// Windows : `AppsUseLightTheme` sous les préférences de personnalisation.
#[cfg(windows)]
async fn detecter_plateforme() -> Option<bool> {
    tokio::task::spawn_blocking(|| {
        let cle = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
            .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
            .ok()?;
        let clair: u32 = cle.get_value("AppsUseLightTheme").ok()?;
        Some(clair == 0)
    })
    .await
    .ok()
    .flatten()
}

/// Plateformes sans détection implémentée : le thème courant est conservé.
#[cfg(not(any(target_os = "linux", windows)))]
async fn detecter_plateforme() -> Option<bool> {
    None
}

/// Résout la préférence de l'utilisateur en thème effectif.
///
/// `systeme` est le dernier résultat connu de [`detecter`] ; `courant` sert de repli quand le
/// système ne s'est pas prononcé.
#[must_use]
pub const fn resoudre(
    pref: crate::modules::settings::model::ThemePref,
    systeme: Option<bool>,
    courant: bool,
) -> bool {
    use crate::modules::settings::model::ThemePref;
    match pref {
        ThemePref::Light => false,
        ThemePref::Dark => true,
        ThemePref::System => match systeme {
            Some(sombre) => sombre,
            None => courant,
        },
    }
}

#[cfg(test)]
#[path = "tests/theme_systeme/mod.rs"]
mod tests;
