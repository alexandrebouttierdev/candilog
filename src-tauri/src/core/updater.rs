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
        // Les hôtes d'assets servent les releases de **tous** les dépôts GitHub : les
        // accepter sans condition revenait à n'en filtrer aucun. Le chemin redirigé porte
        // toujours le dépôt d'origine.
        Some("objects.githubusercontent.com")
        | Some("release-assets.githubusercontent.com")
        | Some("github-releases.githubusercontent.com") => parsed.path().contains(&prefix),
        _ => false,
    }
}

/// Nom de l'asset portant les empreintes SHA-256 de la release, publié par le workflow.
pub const CHECKSUMS_ASSET: &str = "SHA256SUMS";

/// Empreinte hexadécimale SHA-256 d'un contenu.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .fold(String::with_capacity(64), |mut sortie, octet| {
            use std::fmt::Write;
            let _ = write!(sortie, "{octet:02x}");
            sortie
        })
}

/// Lit l'empreinte attendue d'un fichier dans un `SHA256SUMS` au format `sha256sum`.
#[must_use]
pub fn empreinte_attendue(sommes: &str, name_file: &str) -> Option<String> {
    sommes.lines().find_map(|row| {
        let (empreinte, nom) = row.split_once(char::is_whitespace)?;
        // `sha256sum` préfixe le nom d'un `*` en mode binaire.
        let nom = nom.trim().trim_start_matches('*');
        (nom == name_file && empreinte.len() == 64).then(|| empreinte.to_lowercase())
    })
}

/// Compare le contenu téléchargé à l'empreinte publiée.
///
/// # Errors
/// Retourne `Validation` si les empreintes diffèrent — le paquet ne doit alors pas être ouvert.
pub fn verifier_empreinte(bytes: &[u8], attendue: &str) -> AppResult<()> {
    if sha256_hex(bytes).eq_ignore_ascii_case(attendue.trim()) {
        Ok(())
    } else {
        Err(AppError::Validation(
            "L'empreinte du paquet téléchargé ne correspond pas à celle publiée par la release. \
             Le fichier a été supprimé."
                .into(),
        ))
    }
}

/// Refuse un nom d'installateur qui ne porte pas l'extension attendue par la plateforme.
///
/// # Errors
/// Retourne `Validation` si l'extension ne correspond pas : le lanceur système ouvrirait
/// sinon le fichier avec une autre application que le gestionnaire de paquets.
pub fn assert_nom_installeur(name: &str) -> AppResult<()> {
    let Some(extension) = extension_installeur() else {
        return Err(AppError::Validation(
            "Aucun installateur Candilog n'est publié pour ce système.".into(),
        ));
    };
    if std::path::Path::new(name)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
    {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "L'installateur attendu pour ce système porte l'extension .{extension}."
        )))
    }
}

/// Chemin libre dans `dossier`, en suffixant `-1`, `-2`… tant qu'un homonyme existe.
///
/// Le nom vient de la release : écraser un fichier déjà présent dans le dossier
/// Téléchargements détruirait une donnée de l'utilisateur sans le prévenir.
#[must_use]
pub fn chemin_libre(dossier: &Path, name_file: &str) -> PathBuf {
    let candidat = dossier.join(name_file);
    if !candidat.exists() {
        return candidat;
    }
    let base = Path::new(name_file);
    let racine = base
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(name_file);
    let extension = base.extension().and_then(|value| value.to_str());
    for index in 1_u32..1_000 {
        let nom = match extension {
            Some(extension) => format!("{racine}-{index}.{extension}"),
            None => format!("{racine}-{index}"),
        };
        let candidat = dossier.join(nom);
        if !candidat.exists() {
            return candidat;
        }
    }
    dossier.join(format!("{racine}-{}", uuid::Uuid::new_v4()))
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
    /// Tous les assets publiés, `SHA256SUMS` compris.
    ///
    /// Conservés ici et non exposés à l'IPC : le téléchargement les relit côté Rust pour
    /// retrouver l'empreinte attendue, sans jamais dépendre de ce que le frontend renvoie.
    pub assets: Vec<AssetInfo>,
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
    parse_response(&text, current)
}

/// Décode la réponse de l'API GitHub et la compare à la version locale, hors réseau.
///
/// # Errors
/// Retourne `Provider` si la réponse ne respecte pas le contrat attendu ou si son tag
/// n'est pas une version sémantique valide.
pub fn parse_response(json: &str, current: &Version) -> AppResult<Option<UpdateInfo>> {
    let release: ReleaseApi = serde_json::from_str(json).map_err(|error| {
        tracing::warn!(%error, "réponse de mise à jour illisible");
        AppError::Provider("La réponse du service de mise à jour est illisible.".into())
    })?;
    let tag = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
    let remote = Version::parse(tag).map_err(|error| {
        tracing::warn!(tag, %error, "version de mise à jour invalide");
        AppError::Provider("La version publiée par le service de mise à jour est invalide.".into())
    })?;
    if remote <= *current {
        return Ok(None);
    }
    let assets: Vec<AssetInfo> = release
        .assets
        .into_iter()
        .map(|asset| AssetInfo {
            name: asset.name,
            url: asset.browser_download_url,
        })
        .collect();
    Ok(Some(UpdateInfo {
        version: remote,
        notes: release.body,
        page_url: release.html_url,
        asset: asset_pour_plateforme(&assets),
        assets,
    }))
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

/// Télécharge le fichier d'empreintes de la release et en extrait celle de l'asset visé.
///
/// # Errors
/// Retourne `Validation` si la release ne publie pas d'empreinte pour ce fichier : sans
/// elle, rien ne distingue le paquet officiel d'un paquet substitué, et l'installateur ne
/// doit pas être lancé.
async fn empreinte_de_la_release(
    client: &reqwest::Client,
    assets: &[AssetInfo],
    name_file: &str,
) -> AppResult<String> {
    let sommes = assets
        .iter()
        .find(|asset| asset.name == CHECKSUMS_ASSET)
        .ok_or_else(|| {
            AppError::Validation(format!(
                "Cette release ne publie pas de fichier {CHECKSUMS_ASSET} : l'installateur ne peut pas être vérifié."
            ))
        })?;
    assert_url_installeur_autorisee(&sommes.url)?;
    let corps = client
        .get(&sommes.url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    empreinte_attendue(&corps, name_file).ok_or_else(|| {
        AppError::Validation(format!(
            "Aucune empreinte publiée pour {name_file} : l'installateur ne peut pas être vérifié."
        ))
    })
}

/// Télécharge l'installeur dans le dossier Téléchargements, après vérification de son empreinte.
///
/// Le paquet est retenu en mémoire jusqu'à la comparaison : il est plafonné à
/// [`MAX_UPDATE_BYTES`], et écrire d'abord sur disque laisserait exister, le temps de la
/// vérification, un fichier que le lanceur système pourrait ouvrir.
///
/// # Errors
/// Retourne une erreur de réseau, d'écriture, de taille ou d'empreinte.
pub async fn download_installeur(
    client: &reqwest::Client,
    assets: &[AssetInfo],
    url: &str,
    name_file: &str,
    mut on_progress: impl FnMut(u8),
) -> AppResult<PathBuf> {
    assert_url_installeur_autorisee(url)?;
    let name_file = nom_de_fichier_sur(name_file);
    assert_nom_installeur(&name_file)?;
    let attendue = empreinte_de_la_release(client, assets, &name_file).await?;

    let response = client.get(url).send().await?.error_for_status()?;
    let total = response.content_length();
    if let Some(length) = total {
        check_size_paquet(length)?;
    }
    let mut paquet: Vec<u8> = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        check_size_paquet(paquet.len().saturating_add(chunk.len()) as u64)?;
        paquet.extend_from_slice(&chunk);
        if let Some(total) = total {
            let percentage = (paquet.len() as u64)
                .saturating_mul(100)
                .checked_div(total)
                .unwrap_or_default();
            on_progress(u8::try_from(percentage.min(100)).unwrap_or(100));
        }
    }
    verifier_empreinte(&paquet, &attendue)?;

    // `create_new` : un homonyme déjà présent dans le dossier Téléchargements appartient à
    // l'utilisateur et n'a pas à être remplacé par une mise à jour.
    let path = chemin_libre(&dossier_telechargements(), &name_file);
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|error| {
            AppError::Validation(format!(
                "L'installeur n'a pas pu être créé dans le dossier Téléchargements : {error}"
            ))
        })?;
    if let Err(error) = file.write_all(&paquet).await {
        drop(file);
        let _ = tokio::fs::remove_file(&path).await;
        return Err(AppError::Validation(format!(
            "L'installeur n'a pas pu être écrit dans le dossier Téléchargements : {error}"
        )));
    }
    file.sync_all().await.map_err(|error| {
        AppError::Validation(format!(
            "L'installeur n'a pas pu être enregistré sur le disque : {error}"
        ))
    })?;
    on_progress(100);
    tracing::info!(path = %path.display(), "installeur téléchargé et vérifié");
    Ok(path)
}

/// Ouvre un fichier téléchargé avec le lanceur par défaut du système.
///
/// Passe par `tauri-plugin-opener`, déjà utilisé par l'application, plutôt que par une
/// commande construite à la main. Sur Windows, `cmd /C start` réinterprète `&`, `^` et `%`
/// après l'échappement d'arguments de Rust : un chemin aussi banal que
/// `C:\Users\A&B\Downloads\candilog.exe` — une esperluette est légale dans un nom de
/// compte — coupait la commande au milieu. Le plugin cite l'argument, et l'ouverture ne
/// dépend plus de la ponctuation du chemin (`docs/CODE_RULES.md` §14).
///
/// # Errors
/// Retourne une erreur si le lanceur système refuse de démarrer.
pub fn ouvrir_file(path: &Path) -> AppResult<()> {
    tauri_plugin_opener::open_path(path, None::<&str>).map_err(|error| {
        AppError::Validation(format!("Lancement de l'installeur impossible : {error}"))
    })
}

/// Ouvre la page web d'une release dans le navigateur par défaut.
///
/// # Errors
/// Retourne une erreur si le lanceur système refuse de démarrer.
pub fn ouvrir_page(url: &str) -> AppResult<()> {
    tauri_plugin_opener::open_url(url, None::<&str>)
        .map_err(|error| AppError::Validation(format!("Ouverture de la page impossible : {error}")))
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
            .expect("la réponse valide ne doit pas produire d'erreur")
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
        assert_eq!(
            parse_response(response, &Version::new(0, 2, 0)).unwrap(),
            None
        );
        assert_eq!(
            parse_response(response, &Version::new(0, 3, 0)).unwrap(),
            None
        );
    }

    #[test]
    fn un_json_incomplet_ou_invalide_est_refuse() {
        let current = Version::new(0, 2, 0);
        assert!(matches!(
            parse_response("pas du json", &current),
            Err(AppError::Provider(_))
        ));
        assert!(matches!(
            parse_response("{}", &current),
            Err(AppError::Provider(_))
        ));
        assert!(matches!(
            parse_response(
                r#"{"tag_name":"version-invalide","html_url":"https://example.test","assets":[]}"#,
                &current
            ),
            Err(AppError::Provider(_))
        ));
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

    /// L'installateur est lancé par le système : sans empreinte publiée et vérifiée, HTTPS
    /// et l'allowlist d'hôtes sont les seules garanties, et aucune des deux ne dit ce que
    /// contient le fichier reçu.
    #[test]
    fn l_empreinte_attendue_est_lue_dans_le_fichier_de_sommes() {
        let sommes = "\
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  candilog-ubuntu-0.3.0.deb
5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03  candilog-fedora-0.3.0.rpm
";
        assert_eq!(
            empreinte_attendue(sommes, "candilog-fedora-0.3.0.rpm").as_deref(),
            Some("5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03")
        );
        assert_eq!(empreinte_attendue(sommes, "candilog-absent.deb"), None);
    }

    /// Une empreinte qui ne correspond pas doit arrêter la chaîne avant l'ouverture du
    /// fichier — c'est le seul point où un paquet substitué peut encore être refusé.
    #[test]
    fn une_empreinte_divergente_refuse_le_paquet() {
        let vide = sha256_hex(b"");
        assert_eq!(
            vide,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert!(verifier_empreinte(b"", &vide).is_ok());
        assert!(matches!(
            verifier_empreinte(b"contenu substitue", &vide),
            Err(AppError::Validation(_))
        ));
    }

    /// L'allowlist acceptait n'importe quel chemin sur les hôtes d'assets GitHub, qui
    /// servent les releases de **tous** les dépôts.
    #[test]
    fn un_asset_d_un_autre_depot_est_refuse() {
        assert!(!url_installeur_autorisee(
            "https://objects.githubusercontent.com/autre/depot/malware.deb"
        ));
        assert!(url_installeur_autorisee(
            "https://objects.githubusercontent.com/github-production-release-asset/alexandrebouttierdev/candilog/candilog.deb"
        ));
    }

    /// Le nom vient de la release : rien n'empêchait d'écraser un fichier homonyme déjà
    /// présent dans le dossier Téléchargements de l'utilisateur.
    #[test]
    fn un_homonyme_existant_n_est_jamais_ecrase() {
        let directory = tempfile::tempdir().unwrap();
        let occupe = directory.path().join("candilog.deb");
        std::fs::write(&occupe, b"fichier de l'utilisateur").unwrap();

        let libre = chemin_libre(directory.path(), "candilog.deb");

        assert_ne!(libre, occupe);
        assert_eq!(
            std::fs::read(&occupe).unwrap(),
            b"fichier de l'utilisateur",
            "le fichier existant a été touché"
        );
        assert_eq!(
            libre.file_name().and_then(|n| n.to_str()),
            Some("candilog-1.deb")
        );
    }

    /// Le nom du fichier doit porter l'extension d'installateur de la plateforme : un
    /// paquet enregistré sous une autre extension serait ouvert par une autre application.
    #[test]
    fn un_nom_sans_extension_d_installateur_est_refuse() {
        // `extension_installeur()` lit `/etc/os-release` : elle ne renvoie rien sur une
        // distribution hors Debian et Red Hat. Un `expect` y aurait fait échouer la suite
        // de tests d'un contributeur sur Arch, openSUSE ou NixOS, pour un défaut qui
        // n'existe pas.
        let Some(extension) = extension_installeur() else {
            assert!(
                matches!(
                    assert_nom_installeur("candilog.deb"),
                    Err(AppError::Validation(_))
                ),
                "sans installateur publié pour ce système, aucun nom n'est acceptable"
            );
            return;
        };
        assert!(assert_nom_installeur(&format!("candilog.{extension}")).is_ok());
        assert!(matches!(
            assert_nom_installeur("candilog.sh"),
            Err(AppError::Validation(_))
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

    /// Noms d'assets produits par `.github/workflows/release.yml`, étape « Préparer les
    /// assets renommés » : `copy_pair` écrit `candilog-${slug}-${VERSION}.${ext}` et son
    /// jumeau `-latest`, pour les couples (`ubuntu`, `deb`), (`fedora`, `rpm`),
    /// (`windows`, `exe`) et (`macos`, `dmg`).
    fn assets_du_workflow(version: &str) -> Vec<AssetInfo> {
        let mut assets = vec![AssetInfo {
            name: CHECKSUMS_ASSET.to_owned(),
            url: format!(
                "https://github.com/{RELEASES_REPO}/releases/download/v{version}/SHA256SUMS"
            ),
        }];
        for (slug, extension) in [
            ("ubuntu", "deb"),
            ("fedora", "rpm"),
            ("windows", "exe"),
            ("macos", "dmg"),
        ] {
            for suffixe in [version, "latest"] {
                let name = format!("candilog-{slug}-{suffixe}.{extension}");
                assets.push(AssetInfo {
                    url: format!(
                        "https://github.com/{RELEASES_REPO}/releases/download/v{version}/{name}"
                    ),
                    name,
                });
            }
        }
        assets
    }

    /// Contrat de nommage entre le workflow de release et l'application.
    ///
    /// Les autres cas de ce module écrivent leurs noms d'assets à la main : renommer un
    /// asset dans le workflow ne cassait donc aucun test, et le défaut n'aurait été visible
    /// qu'au premier utilisateur cherchant une mise à jour — qui n'aurait rien trouvé.
    #[test]
    fn les_assets_du_workflow_sont_ceux_que_l_application_attend() {
        let assets = assets_du_workflow("1.2.3");
        for (extension, attendu) in [
            ("deb", "candilog-ubuntu-1.2.3.deb"),
            ("rpm", "candilog-fedora-1.2.3.rpm"),
            ("exe", "candilog-windows-1.2.3.exe"),
            ("dmg", "candilog-macos-1.2.3.dmg"),
        ] {
            let choisi = asset_pour_extension(&assets, extension)
                .unwrap_or_else(|| panic!("aucun asset .{extension} reconnu"));
            assert_eq!(choisi.name, attendu);
            assert!(
                url_installeur_autorisee(&choisi.url),
                "l'URL publiée par le workflow doit passer l'allowlist"
            );
        }
        assert!(
            assets.iter().any(|asset| asset.name == CHECKSUMS_ASSET),
            "sans SHA256SUMS l'application refuse d'ouvrir l'installateur"
        );
    }

    /// Le site pointe sur les noms `-latest` : ils doivent rester reconnus, sinon une
    /// release sans copie versionnée laisserait l'application sans installateur.
    #[test]
    fn le_jumeau_latest_reste_reconnu_seul() {
        let assets: Vec<AssetInfo> = assets_du_workflow("1.2.3")
            .into_iter()
            .filter(|asset| asset.name.contains("-latest.") || asset.name == CHECKSUMS_ASSET)
            .collect();
        let choisi = asset_pour_extension(&assets, "deb").expect("deb attendu");
        assert_eq!(choisi.name, "candilog-ubuntu-latest.deb");
    }

    /// Le plafond doit être appliqué au fil du flux, et pas seulement à l'annonce du
    /// serveur : un serveur hostile peut mentir sur `Content-Length`, ou ne pas l'envoyer.
    #[test]
    fn le_plafond_arrete_un_flux_qui_depasse_sans_content_length() {
        let mut recu: u64 = 0;
        let morceau = 64 * 1024 * 1024_u64;
        let mut refuse = false;
        for _ in 0..8 {
            recu = recu.saturating_add(morceau);
            if check_size_paquet(recu).is_err() {
                refuse = true;
                break;
            }
        }
        assert!(refuse, "le cumul des morceaux doit finir par être refusé");
        assert!(
            recu <= MAX_UPDATE_BYTES + morceau,
            "le refus doit intervenir dès le dépassement, pas après le téléchargement entier"
        );
    }

    /// Une release sans `SHA256SUMS`, ou dont le fichier n'a pas de ligne pour l'asset,
    /// ne permet aucune vérification : l'installateur ne doit alors pas être ouvert.
    #[test]
    fn une_empreinte_absente_ne_peut_pas_etre_lue() {
        let sommes = "aa".repeat(32) + "  candilog-ubuntu-1.2.3.deb\n";
        assert!(empreinte_attendue(&sommes, "candilog-windows-1.2.3.exe").is_none());
        assert!(empreinte_attendue("", "candilog-ubuntu-1.2.3.deb").is_none());
        assert!(empreinte_attendue(&sommes, "candilog-ubuntu-1.2.3.deb").is_some());
    }
}
