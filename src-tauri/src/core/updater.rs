//! Vérification native des mises à jour publiées sur GitHub.
//!
//! Candilog interroge l'API GitHub du dépôt public `candilog`. Un `User-Agent` est
//! obligatoire : GitHub refuse les clients anonymes sans identifiant.

use crate::core::errors::{AppError, AppResult};
use futures_util::StreamExt;
use semver::Version;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// Dépôt public GitHub hébergeant le code et les releases de Candilog.
pub const RELEASES_REPO: &str = "alexandrebouttierdev/candilog";
/// API de la dernière release publiée sur ce dépôt.
pub const RELEASES_API_URL: &str =
    "https://api.github.com/repos/alexandrebouttierdev/candilog/releases/latest";
/// Page web de la release, ouverte en dernier recours quand aucun asset ne correspond.
pub const RELEASES_PAGE_URL: &str =
    "https://github.com/alexandrebouttierdev/candilog/releases/latest";

/// Accepte uniquement les URL HTTPS du dépôt officiel de releases.
#[must_use]
pub fn url_installeur_autorisee(url: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url) else {
        return false;
    };
    if parsed.scheme() != "https" {
        return false;
    }
    let prefix = format!("/{RELEASES_REPO}/");
    match parsed.host_str() {
        Some("github.com") => parsed.path().starts_with(&prefix),
        Some("objects.githubusercontent.com")
        | Some("release-assets.githubusercontent.com")
        | Some("github-releases.githubusercontent.com") => true,
        _ => false,
    }
}

/// # Errors
/// Retourne `Validation` si l'URL n'appartient pas au dépôt de releases.
pub fn assert_url_installeur_autorisee(url: &str) -> AppResult<()> {
    if url_installeur_autorisee(url) {
        Ok(())
    } else {
        Err(AppError::Validation(
            "L'adresse de téléchargement n'est pas une release officielle Candilog.".into(),
        ))
    }
}

/// Résultat d'une vérification de mise à jour, avant conversion IPC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    pub version: Version,
    pub notes: String,
    pub page_url: String,
    pub asset: Option<AssetInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetInfo {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
struct ReleaseApi {
    tag_name: String,
    #[serde(default)]
    body: String,
    html_url: String,
    assets: Vec<AssetApi>,
}

#[derive(Debug, Deserialize)]
struct AssetApi {
    name: String,
    browser_download_url: String,
}

/// Client HTTP identifié, exigé par l'API GitHub.
///
/// # Errors
/// Retourne une erreur si le client ne peut pas être construit.
pub fn client_github() -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(format!("Candilog/{}", env!("CARGO_PKG_VERSION")))
        .connect_timeout(std::time::Duration::from_secs(8))
        .timeout(std::time::Duration::from_secs(60 * 5))
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
        .map_err(AppError::from)
}

/// Vérifie si une version plus récente existe pour la plateforme courante.
///
/// # Errors
/// Retourne une erreur lisible si l'API est inaccessible ou illisible.
pub async fn check(client: &reqwest::Client, current: &Version) -> AppResult<Option<UpdateInfo>> {
    check_url(client, current, RELEASES_API_URL).await
}

async fn check_url(
    client: &reqwest::Client,
    current: &Version,
    url: &str,
) -> AppResult<Option<UpdateInfo>> {
    let response = client.get(url).send().await?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let response = response.error_for_status()?;
    let text = response.text().await?;
    Ok(parse_response(&text, current))
}

/// Décode la réponse de l'API GitHub et la compare à la version locale, hors réseau.
#[must_use]
pub fn parse_response(json: &str, current: &Version) -> Option<UpdateInfo> {
    let release: ReleaseApi = serde_json::from_str(json).ok()?;
    let tag = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
    let remote = Version::parse(tag).ok()?;
    if remote <= *current {
        return None;
    }
    let assets: Vec<AssetInfo> = release
        .assets
        .into_iter()
        .map(|asset| AssetInfo {
            name: asset.name,
            url: asset.browser_download_url,
        })
        .collect();
    Some(UpdateInfo {
        version: remote,
        notes: release.body,
        page_url: release.html_url,
        asset: asset_pour_plateforme(&assets),
    })
}

#[must_use]
pub fn asset_pour_plateforme(assets: &[AssetInfo]) -> Option<AssetInfo> {
    let extension = extension_installeur()?;
    asset_pour_extension(assets, extension)
}

#[must_use]
pub fn asset_pour_extension(assets: &[AssetInfo], extension: &str) -> Option<AssetInfo> {
    let suffixe = format!(".{extension}");
    // Préférer le nom versionné (`candilog-ubuntu-0.3.0.deb`) au jumeau `-latest`.
    assets
        .iter()
        .find(|asset| asset.name.ends_with(&suffixe) && !asset.name.contains("-latest."))
        .or_else(|| assets.iter().find(|asset| asset.name.ends_with(&suffixe)))
        .cloned()
}

#[must_use]
pub fn extension_installeur() -> Option<&'static str> {
    if cfg!(target_os = "windows") {
        return extension_pour("windows", &[]);
    }
    if cfg!(target_os = "macos") {
        return extension_pour("macos", &[]);
    }
    if cfg!(target_os = "linux") {
        let ids = lire_os_release_ids();
        let referents: Vec<&str> = ids.iter().map(String::as_str).collect();
        return extension_pour("linux", &referents);
    }
    None
}

#[must_use]
pub fn extension_pour(os: &str, ids: &[&str]) -> Option<&'static str> {
    match os {
        "windows" => Some("exe"),
        "macos" => Some("dmg"),
        "linux" => {
            let debian = ids.iter().any(|id| *id == "debian" || *id == "ubuntu");
            let redhat = ids
                .iter()
                .any(|id| *id == "fedora" || *id == "rhel" || *id == "centos");
            if debian {
                Some("deb")
            } else if redhat {
                Some("rpm")
            } else {
                None
            }
        }
        _ => None,
    }
}

fn lire_os_release_ids() -> Vec<String> {
    let mut ids = Vec::new();
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for row in content.lines() {
            if let Some((cle, value)) = row.split_once('=') {
                if cle == "ID" || cle == "ID_LIKE" {
                    let value = value.trim_matches('"');
                    for id in value.split_whitespace() {
                        ids.push(id.to_lowercase());
                    }
                }
            }
        }
    }
    ids
}

/// Size maximale acceptée pour un installeur natif (256 MiB).
pub const MAX_UPDATE_BYTES: u64 = 256 * 1024 * 1024;

/// Vérifie que la taille d'un installeur ne dépasse pas la limite autorisée.
///
/// # Errors
/// Retourne `Validation` si l'installeur dépasse la taille maximale.
pub fn check_size_paquet(length: u64) -> AppResult<()> {
    if length > MAX_UPDATE_BYTES {
        return Err(AppError::Validation(
            "Le paquet de mise à jour dépasse la taille maximale autorisée.".into(),
        ));
    }
    Ok(())
}

fn dossier_telechargements() -> PathBuf {
    dirs::download_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
}

#[must_use]
pub fn nom_de_fichier_sur(name: &str) -> String {
    let nettoye: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            _ => c,
        })
        .collect();
    if nettoye.trim().is_empty() || nettoye == "." || nettoye == ".." {
        "candilog-installateur".to_string()
    } else {
        nettoye
    }
}

/// Télécharge l'installeur dans le dossier Téléchargements.
///
/// # Errors
/// Retourne une erreur de réseau ou d'écriture. Un fichier trop volumineux est supprimé.
pub async fn download_installeur(
    client: &reqwest::Client,
    url: &str,
    name_file: &str,
    mut on_progress: impl FnMut(u8),
) -> AppResult<PathBuf> {
    assert_url_installeur_autorisee(url)?;
    let response = client.get(url).send().await?.error_for_status()?;
    let total = response.content_length();
    if let Some(length) = total {
        check_size_paquet(length)?;
    }
    let path = dossier_telechargements().join(nom_de_fichier_sur(name_file));
    let mut file = tokio::fs::File::create(&path).await.map_err(|error| {
        AppError::Database(format!("Création de l'installeur impossible : {error}"))
    })?;
    let mut recu = 0_u64;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Err(error) = check_size_paquet(recu.saturating_add(chunk.len() as u64)) {
            drop(file);
            let _ = tokio::fs::remove_file(&path).await;
            return Err(error);
        }
        file.write_all(&chunk).await.map_err(|error| {
            AppError::Database(format!("Écriture de l'installeur impossible : {error}"))
        })?;
        recu = recu.saturating_add(u64::try_from(chunk.len()).unwrap_or_default());
        if let Some(total) = total {
            let percentage = recu
                .saturating_mul(100)
                .checked_div(total)
                .unwrap_or_default();
            on_progress(u8::try_from(percentage.min(100)).unwrap_or(100));
        }
    }
    file.sync_all().await.map_err(|error| {
        AppError::Database(format!(
            "Synchronisation de l'installeur impossible : {error}"
        ))
    })?;
    on_progress(100);
    tracing::info!(path = %path.display(), "installeur téléchargé");
    Ok(path)
}

fn ouvrir_avec_lanceur(cible: &str) -> AppResult<()> {
    #[cfg(target_os = "linux")]
    let (programme, arguments): (&str, Vec<&str>) = ("xdg-open", vec![cible]);
    #[cfg(target_os = "macos")]
    let (programme, arguments): (&str, Vec<&str>) = ("open", vec![cible]);
    #[cfg(target_os = "windows")]
    let (programme, arguments): (&str, Vec<&str>) = ("cmd", vec!["/C", "start", "", cible]);

    std::process::Command::new(programme)
        .args(arguments)
        .spawn()
        .map(|_| ())
        .map_err(|error| {
            AppError::Validation(format!("Lancement de l'installeur impossible : {error}"))
        })
}

/// Ouvre un fichier téléchargé avec le lanceur par défaut du système.
///
/// # Errors
/// Retourne une erreur si le lanceur système refuse de démarrer.
pub fn ouvrir_file(path: &Path) -> AppResult<()> {
    ouvrir_avec_lanceur(&path.to_string_lossy())
}

/// Ouvre la page web d'une release dans le navigateur par défaut.
///
/// # Errors
/// Retourne une erreur si le lanceur système refuse de démarrer.
pub fn ouvrir_page(url: &str) -> AppResult<()> {
    ouvrir_avec_lanceur(url)
}

/// Version locale, lue depuis le manifeste Cargo.
///
/// # Errors
/// Retourne une erreur si le numéro de version du manifeste n'est pas semver.
pub fn version_locale() -> AppResult<Version> {
    Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|error| AppError::Validation(format!("Version locale illisible : {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RESPONSE_EXEMPLE: &str = r#"{
    "tag_name": "v0.3.0",
    "body": "Nouvelle version avec corrections.",
    "html_url": "https://github.com/alexandrebouttierdev/candilog/releases/tag/v0.3.0",
    "assets": [
        {
            "name": "candilog-ubuntu-latest.deb",
            "browser_download_url": "https://github.com/.../candilog-ubuntu-latest.deb"
        },
        {
            "name": "candilog-ubuntu-0.3.0.deb",
            "browser_download_url": "https://github.com/.../candilog-ubuntu-0.3.0.deb"
        },
        {
            "name": "candilog-fedora-0.3.0.rpm",
            "browser_download_url": "https://github.com/.../candilog-fedora-0.3.0.rpm"
        },
        {
            "name": "candilog-macos-0.3.0.dmg",
            "browser_download_url": "https://github.com/.../candilog-macos-0.3.0.dmg"
        },
        {
            "name": "candilog-windows-0.3.0.exe",
            "browser_download_url": "https://github.com/.../candilog-windows-0.3.0.exe"
        }
    ]
}"#;

    #[test]
    fn une_reponse_github_complete_est_decodee() {
        let info = parse_response(RESPONSE_EXEMPLE, &Version::new(0, 2, 0))
            .expect("la réponse complète doit être décodée");
        assert_eq!(info.version, Version::new(0, 3, 0));
        assert_eq!(info.notes, "Nouvelle version avec corrections.");
        if let Some(asset) = info.asset {
            let extension = extension_installeur().expect("plateforme reconnue en test");
            assert!(asset.name.ends_with(&format!(".{extension}")));
        }
    }

    #[test]
    fn une_version_egale_ou_inferieure_est_ignoree() {
        let response = r#"{"tag_name":"v0.2.0","html_url":"https://example.test","assets":[]}"#;
        assert_eq!(parse_response(response, &Version::new(0, 2, 0)), None);
        assert_eq!(parse_response(response, &Version::new(0, 3, 0)), None);
    }

    #[test]
    fn un_json_incomplet_ou_invalide_est_refuse() {
        let actuelle = Version::new(0, 2, 0);
        assert_eq!(parse_response("pas du json", &actuelle), None);
        assert_eq!(parse_response("{}", &actuelle), None);
    }

    #[test]
    fn la_famille_linux_choisit_deb_ou_rpm() {
        assert_eq!(extension_pour("windows", &[]), Some("exe"));
        assert_eq!(extension_pour("macos", &[]), Some("dmg"));
        assert_eq!(extension_pour("linux", &["debian"]), Some("deb"));
        assert_eq!(extension_pour("linux", &["ubuntu"]), Some("deb"));
        assert_eq!(extension_pour("linux", &["fedora"]), Some("rpm"));
        assert_eq!(extension_pour("linux", &["arch"]), None);
        assert_eq!(extension_pour("plan9", &[]), None);
    }

    #[test]
    fn les_caracteres_dangereux_sont_remplaces() {
        assert_eq!(
            nom_de_fichier_sur("a/b\\c:d*e?f\"g<h>i|j"),
            "a_b_c_d_e_f_g_h_i_j"
        );
        assert_eq!(nom_de_fichier_sur(".."), "candilog-installateur");
    }

    #[test]
    fn download_refuse_un_paquet_trop_volumineux() {
        check_size_paquet(MAX_UPDATE_BYTES).unwrap();
        assert!(check_size_paquet(MAX_UPDATE_BYTES + 1).is_err());
    }

    #[test]
    fn seules_les_url_du_depot_officiel_sont_acceptees() {
        assert!(url_installeur_autorisee(
            "https://github.com/alexandrebouttierdev/candilog/releases/download/v0.3.0/candilog.deb"
        ));
        assert!(!url_installeur_autorisee(
            "https://evil.example/malware.exe"
        ));
        assert!(!url_installeur_autorisee(
            "http://github.com/alexandrebouttierdev/candilog/x"
        ));
        assert!(!url_installeur_autorisee(
            "https://github.com/autre/depot/releases/download/v1/x.deb"
        ));
        assert!(!url_installeur_autorisee(
            "https://github.com/alexandrebouttierdev/candilog-releases/releases/download/v0.3.0/candilog.deb"
        ));
    }

    #[test]
    fn l_asset_versionne_est_prefere_au_jumeau_latest() {
        let assets = [
            AssetInfo {
                name: "candilog-ubuntu-latest.deb".into(),
                url: "https://example.test/latest.deb".into(),
            },
            AssetInfo {
                name: "candilog-ubuntu-0.3.0.deb".into(),
                url: "https://example.test/0.3.0.deb".into(),
            },
        ];
        let choisi = asset_pour_extension(&assets, "deb").expect("deb attendu");
        assert_eq!(choisi.name, "candilog-ubuntu-0.3.0.deb");
    }
}
