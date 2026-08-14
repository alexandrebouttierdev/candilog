//! Vérification native des mises à jour publiées sur GitHub.
//!
//! Candilog interroge l'API GitHub du dépôt public `candilog-releases`, distinct du dépôt
//! source, qui porte le tag `v<version>` et les assets d'installation de chaque plateforme.
//! Seul l'identifiant GitHub est requis : pas de token pour un dépôt public (la limite
//! anonyme de 60 requêtes/heure suffit pour une vérification à chaque ouverture).

use crate::shared::error::{AppError, AppResult};
use semver::Version;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// Dépôt public GitHub hébergeant les releases de Candilog.
pub const RELEASES_REPO: &str = "alexandrebouttierdev/candilog-releases";
/// API de la dernière release publiée sur ce dépôt.
pub const RELEASES_API_URL: &str =
    "https://api.github.com/repos/alexandrebouttierdev/candilog-releases/releases/latest";
/// Page web de la release, ouverte en dernier recours quand aucun asset ne correspond au
/// système.
pub const RELEASES_PAGE_URL: &str =
    "https://github.com/alexandrebouttierdev/candilog-releases/releases/latest";

/// Résultat d'une vérification de mise à jour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    /// Version distante.
    pub version: Version,
    /// Notes de version.
    pub notes: String,
    /// Page web de la release, ouverte quand aucun asset ne correspond au système.
    pub page_url: String,
    /// Asset d'installation adapté à la plateforme courante, s'il est publié.
    pub asset: Option<AssetInfo>,
}

/// Un asset téléchargeable d'une release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetInfo {
    /// Nom du fichier sur la release.
    pub name: String,
    /// URL de téléchargement.
    pub url: String,
}

/// Réponse de l'API GitHub, réduite aux champs utiles.
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

/// Vérifie si une version plus récente existe pour la plateforme courante.
///
/// # Errors
/// Retourne une erreur lisible si l'API est inaccessible ou illisible.
pub async fn check(client: &reqwest::Client, current: &Version) -> AppResult<Option<UpdateInfo>> {
    check_url(client, current, RELEASES_API_URL).await
}

/// Vérifie la mise à jour auprès d'une URL d'API donnée.
///
/// Variante testable de [`check`] : l'URL est injectée pour intercepter la requête hors
/// réseau.
///
/// # Errors
/// Retourne une erreur lisible si l'API est inaccessible ou illisible.
async fn check_url(
    client: &reqwest::Client,
    current: &Version,
    url: &str,
) -> AppResult<Option<UpdateInfo>> {
    let response = client.get(url).send().await?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        // Aucune release publiée : pas de mise à jour à proposer.
        return Ok(None);
    }
    let response = response.error_for_status()?;
    let texte = response.text().await?;
    Ok(analyser_reponse(&texte, current))
}

/// Décode la réponse de l'API GitHub et la compare à la version locale, hors réseau.
///
/// Fonction pure pour permettre le test hors ligne ; `None` si la réponse ne correspond pas
/// au format attendu (y compris une version invalide) ou si la version distante n'est pas
/// plus récente.
pub fn analyser_reponse(json: &str, current: &Version) -> Option<UpdateInfo> {
    let release: ReleaseApi = serde_json::from_str(json).ok()?;
    // Les tags GitHub sont conventionnellement préfixés par `v` (`v0.3.0`), ce que le crate
    // `semver` refuse de parser : on retire ce préfixe avant comparaison.
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

/// Asset d'installation de la plateforme courante, si le système est reconnu.
pub fn asset_pour_plateforme(assets: &[AssetInfo]) -> Option<AssetInfo> {
    let extension = extension_installeur()?;
    asset_pour_extension(assets, extension)
}

/// Premier asset dont le nom se termine par `.extension`.
pub fn asset_pour_extension(assets: &[AssetInfo], extension: &str) -> Option<AssetInfo> {
    let suffixe = format!(".{extension}");
    assets
        .iter()
        .find(|asset| asset.name.ends_with(&suffixe))
        .cloned()
}

/// Extension d'asset d'installation à télécharger pour ce système, sans le point.
///
/// Retourne `None` pour un système non reconnu : l'utilisateur est alors renvoyé vers la page
/// de la release.
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

/// Extension correspondant à un couple système/famille, sans le point.
///
/// Fonction pure, testable indépendamment de la machine : `os` vaut `"windows"`, `"macos"`
/// ou `"linux"` ; `ids` porte les identifiants de famille Linux (`ID` et `ID_LIKE` de
/// `/etc/os-release`).
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

/// Identifiants `ID` et `ID_LIKE` du système, en minuscules.
///
/// Se lit uniquement sur Linux ; ailleurs, la liste est vide. Une absence de fichier ou une
/// lecture impossible sont traitées comme une famille inconnue.
fn lire_os_release_ids() -> Vec<String> {
    let mut ids = Vec::new();
    if let Ok(contenu) = std::fs::read_to_string("/etc/os-release") {
        for ligne in contenu.lines() {
            if let Some((cle, valeur)) = ligne.split_once('=') {
                if cle == "ID" || cle == "ID_LIKE" {
                    let valeur = valeur.trim_matches('"');
                    for id in valeur.split_whitespace() {
                        ids.push(id.to_lowercase());
                    }
                }
            }
        }
    }
    ids
}

/// Taille maximale acceptée pour un installeur natif (256 MiB).
pub const MAX_UPDATE_BYTES: u64 = 256 * 1024 * 1024;

/// Vérifie que la taille d'un installeur ne dépasse pas la limite autorisée.
///
/// # Errors
/// Retourne `Validation` si l'installeur dépasse la taille maximale.
pub fn verifier_taille_paquet(length: u64) -> AppResult<()> {
    if length > MAX_UPDATE_BYTES {
        return Err(AppError::Validation(
            "Le paquet de mise à jour dépasse la taille maximale autorisée.".into(),
        ));
    }
    Ok(())
}

/// Dossier où poser l'installeur téléchargé.
fn dossier_telechargements() -> PathBuf {
    dirs::download_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
}

/// Nom de fichier sûr, sans les caractères interdits ou douteux.
///
/// L'asset vient de la release du dépôt ; l'assainissement protège quand même d'un nom
/// inattendu dans un chemin du système de fichiers.
pub fn nom_de_fichier_sur(nom: &str) -> String {
    let nettoye: String = nom
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

/// Télécharge l'installeur **dans le dossier Téléchargements** et retourne son chemin.
///
/// Le fichier allait auparavant sous le dossier de données puis dans `/tmp` : l'un et l'autre
/// sont mal adaptés à un installeur que l'utilisateur doit exécuter lui-même. Le dossier
/// Téléchargements du système est l'emplacement naturel, il n'est pas purgé au redémarrage.
///
/// `on_progress` reçoit un pourcentage borné à 0–100 quand la taille est connue. Le fichier
/// est remplacé s'il existe déjà.
///
/// # Errors
/// Retourne une erreur de réseau ou d'écriture. Un fichier trop volumineux est supprimé.
pub async fn telecharger_installeur(
    client: &reqwest::Client,
    url: &str,
    nom_fichier: &str,
    mut on_progress: impl FnMut(u8),
) -> AppResult<PathBuf> {
    use iced::futures::StreamExt;

    let response = client.get(url).send().await?.error_for_status()?;
    let total = response.content_length();
    if let Some(length) = total {
        verifier_taille_paquet(length)?;
    }
    let dossier = dossier_telechargements();
    let chemin = dossier.join(nom_de_fichier_sur(nom_fichier));
    // `tokio::fs` plutôt que `std::fs` : chaque écriture bloquante immobiliserait un fil de
    // travail du runtime pour la durée de l'E/S, réduisant d'autant le parallélisme
    // disponible aux autres tâches asynchrones.
    let mut fichier = tokio::fs::File::create(&chemin).await.map_err(|error| {
        AppError::Database(format!("Création de l'installeur impossible : {error}"))
    })?;
    let mut recu = 0_u64;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Err(error) = verifier_taille_paquet(recu.saturating_add(chunk.len() as u64)) {
            drop(fichier);
            let _ = tokio::fs::remove_file(&chemin).await;
            return Err(error);
        }
        fichier.write_all(&chunk).await.map_err(|error| {
            AppError::Database(format!("Écriture de l'installeur impossible : {error}"))
        })?;
        recu = recu.saturating_add(u64::try_from(chunk.len()).unwrap_or_default());
        if let Some(total) = total {
            let pourcentage = recu
                .saturating_mul(100)
                .checked_div(total)
                .unwrap_or_default();
            on_progress(u8::try_from(pourcentage.min(100)).unwrap_or(100));
        }
    }
    fichier.sync_all().await.map_err(|error| {
        AppError::Database(format!(
            "Synchronisation de l'installeur impossible : {error}"
        ))
    })?;
    on_progress(100);
    tracing::info!(chemin = %chemin.display(), "installeur téléchargé");
    Ok(chemin)
}

/// Ouvre une cible avec le lanceur par défaut du système.
///
/// Chaque plateforme a son lanceur ; aucune dépendance supplémentaire n'est nécessaire pour
/// les trois cibles visées.
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
pub fn ouvrir_fichier(chemin: &Path) -> AppResult<()> {
    ouvrir_avec_lanceur(&chemin.to_string_lossy())
}

/// Ouvre la page web d'une release dans le navigateur par défaut.
pub fn ouvrir_page(url: &str) -> AppResult<()> {
    ouvrir_avec_lanceur(url)
}

#[cfg(test)]
#[path = "tests/updater/mod.rs"]
mod tests;
