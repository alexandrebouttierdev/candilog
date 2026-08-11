//! Vérification native des mises à jour publiées sur GitLab.

use crate::shared::error::{AppError, AppResult};
use semver::Version;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// URL publique du manifeste de release Candilog.
pub const MANIFEST_URL: &str =
    "https://gitlab.com/alexandrebouttier/candilog-releases/-/raw/main/latest.json";

/// Résultat d'une vérification de mise à jour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    /// Version distante.
    pub version: Version,
    /// Notes de version.
    pub notes: String,
    /// URL du paquet adapté à la plateforme courante.
    pub download_url: String,
    /// Signature minisign publiée avec le paquet.
    pub signature: String,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    version: String,
    #[serde(default)]
    notes: String,
    platforms: std::collections::HashMap<String, Platform>,
}

#[derive(Debug, Deserialize)]
struct Platform {
    url: String,
    signature: String,
}

/// Vérifie si une version plus récente existe pour la plateforme courante.
///
/// # Errors
/// Retourne une erreur lisible si le manifeste est inaccessible ou invalide.
pub async fn check(client: &reqwest::Client, current: &Version) -> AppResult<Option<UpdateInfo>> {
    let response = client.get(MANIFEST_URL).send().await?.error_for_status()?;
    let manifest: Manifest = response.json().await?;
    let remote = Version::parse(&manifest.version)
        .map_err(|error| AppError::Serialization(format!("Version distante invalide : {error}")))?;
    if remote <= *current {
        return Ok(None);
    }
    let platform_key = current_platform_key();
    let platform = manifest.platforms.get(platform_key).ok_or_else(|| {
        AppError::NotFound(format!("Aucun paquet de mise à jour pour {platform_key}"))
    })?;
    Ok(Some(UpdateInfo {
        version: remote,
        notes: manifest.notes,
        download_url: platform.url.clone(),
        signature: platform.signature.clone(),
    }))
}

/// Sous-dossier du dossier de données recevant les paquets téléchargés.
pub const DOSSIER_MISES_A_JOUR: &str = "mises-a-jour";

/// Télécharge un paquet **sous le dossier de données** et vérifie sa signature minisign.
///
/// Le paquet allait auparavant dans `std::env::temp_dir()`. Comme l'installation n'est pas
/// automatisée, l'utilisateur doit s'en occuper lui-même — or de nombreux systèmes purgent
/// `/tmp` au redémarrage : le fichier pouvait avoir disparu avant qu'il n'y arrive.
///
/// `on_progress` reçoit un pourcentage borné à 0–100 quand la taille est connue.
/// Le fichier n'est renvoyé qu'après validation cryptographique.
///
/// # Errors
/// Retourne une erreur de réseau, d'écriture ou de signature. Un fichier invalide est supprimé.
pub async fn download_verified(
    client: &reqwest::Client,
    update: &UpdateInfo,
    destination_dir: &Path,
    mut on_progress: impl FnMut(u8),
) -> AppResult<PathBuf> {
    use iced::futures::StreamExt;

    let response = client
        .get(&update.download_url)
        .send()
        .await?
        .error_for_status()?;
    let total = response.content_length();
    let extension = update
        .download_url
        .rsplit_once('/')
        .map_or("candilog-update.bin", |(_, name)| name);
    let dossier = destination_dir.join(DOSSIER_MISES_A_JOUR);
    tokio::fs::create_dir_all(&dossier).await.map_err(|error| {
        AppError::Database(format!(
            "Création du dossier de mise à jour impossible : {error}"
        ))
    })?;
    let path = dossier.join(format!("candilog-{}-{extension}", update.version));
    // `tokio::fs` plutôt que `std::fs` : chaque écriture bloquante immobiliserait un fil de
    // travail du runtime pour la durée de l'E/S, réduisant d'autant le parallélisme
    // disponible aux autres tâches asynchrones.
    let mut file = tokio::fs::File::create(&path).await.map_err(|error| {
        AppError::Database(format!("Création du téléchargement impossible : {error}"))
    })?;
    let mut received = 0_u64;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await.map_err(|error| {
            AppError::Database(format!("Écriture du téléchargement impossible : {error}"))
        })?;
        received = received.saturating_add(u64::try_from(chunk.len()).unwrap_or_default());
        if let Some(total) = total {
            let percentage = received
                .saturating_mul(100)
                .checked_div(total)
                .unwrap_or_default();
            on_progress(u8::try_from(percentage.min(100)).unwrap_or(100));
        }
    }
    file.sync_all().await.map_err(|error| {
        AppError::Database(format!(
            "Synchronisation du téléchargement impossible : {error}"
        ))
    })?;
    drop(file);
    // La vérification relit le fichier entier et calcule un condensat : travail bloquant et
    // gourmand en CPU, donc confié à un fil dédié.
    let paquet = path.clone();
    let signature = update.signature.clone();
    let verification = tokio::task::spawn_blocking(move || verify_package(&paquet, &signature))
        .await
        .map_err(|error| AppError::Database(format!("Vérification interrompue : {error}")))?;
    if let Err(error) = verification {
        let _ = tokio::fs::remove_file(&path).await;
        tracing::error!(erreur = %error, "signature du paquet invalide, fichier supprimé");
        return Err(error);
    }
    on_progress(100);
    tracing::info!(version = %update.version, "mise à jour téléchargée et vérifiée");
    Ok(path)
}

/// Vérifie un paquet avec la clé publique historique de Candilog.
///
/// # Errors
/// Retourne `Validation` si la signature est mal formée ou ne correspond pas au paquet.
pub fn verify_package(path: &Path, signature_text: &str) -> AppResult<()> {
    const PUBLIC_KEY: &str = "RWQjmHtMlNuNe37jKdRbKek12X7DGia33IFTW6Bm4SG8rbVm8AJpfnYY";
    let public_key = minisign_verify::PublicKey::from_base64(PUBLIC_KEY)
        .map_err(|error| AppError::Validation(format!("Clé de mise à jour invalide : {error}")))?;
    let signature = minisign_verify::Signature::decode(signature_text).map_err(|error| {
        AppError::Validation(format!("Signature de mise à jour invalide : {error}"))
    })?;
    let bytes = std::fs::read(path).map_err(|error| {
        AppError::Database(format!("Lecture de la mise à jour impossible : {error}"))
    })?;
    public_key
        .verify(&bytes, &signature, false)
        .map_err(|error| {
            AppError::Validation(format!(
                "La signature de la mise à jour ne correspond pas : {error}"
            ))
        })
}

fn current_platform_key() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "linux-x86_64";
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "windows-x86_64";
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "darwin-aarch64";
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "darwin-x86_64";
    #[allow(unreachable_code)]
    "unsupported"
}
