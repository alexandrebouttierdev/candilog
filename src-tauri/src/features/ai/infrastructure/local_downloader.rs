//! Téléchargement streamé, reprenable et vérifié des GGUF verrouillés.

use crate::core::errors::{AppError, AppResult};
use crate::features::ai::domain::{LocalAiDownloadProgress, LocalAiState, LocalModelId};
use futures_util::StreamExt;
use reqwest::header::{CONTENT_RANGE, RANGE};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

const DISK_RESERVE_BYTES: u64 = 512 * 1024 * 1024;

pub struct ModelDownloader {
    client: reqwest::Client,
    allow_test_urls: bool,
}

pub struct ModelDownload<'a> {
    pub model_id: LocalModelId,
    pub url: &'a str,
    pub destination: &'a Path,
    pub expected_size: u64,
    pub expected_sha256: &'a str,
}

impl ModelDownloader {
    pub fn new() -> AppResult<Self> {
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(15))
            .timeout(std::time::Duration::from_secs(60 * 60 * 12))
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().len() >= 5 {
                    return attempt.stop();
                }
                if allowed_download_host(attempt.url().host_str()) {
                    attempt.follow()
                } else {
                    attempt.stop()
                }
            }))
            .user_agent(concat!("Candilog/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(AppError::from)?;
        Ok(Self {
            client,
            allow_test_urls: false,
        })
    }

    #[cfg(test)]
    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            client,
            allow_test_urls: true,
        }
    }

    pub async fn download(
        &self,
        request: ModelDownload<'_>,
        cancellation: &CancellationToken,
        progress: impl Fn(LocalAiDownloadProgress),
    ) -> AppResult<PathBuf> {
        let ModelDownload {
            model_id,
            url,
            destination,
            expected_size,
            expected_sha256,
        } = request;
        if !self.allow_test_urls {
            validate_source(url)?;
        }
        validate_destination(destination)?;
        let parent = destination.parent().ok_or_else(|| {
            AppError::Validation("Le dossier du modèle local est invalide.".into())
        })?;
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(file_error)?;
        if cancellation.is_cancelled() {
            // Domaine : LocalAiError::DownloadCancelled. AppError::Cancelled conserve le
            // code CANCELLED pour le frontend ; le message « Génération annulée » n'est
            // pas affiché (install clear + ViewModel toast « Téléchargement annulé »).
            return Err(AppError::Cancelled);
        }
        if valid_existing_model(destination, expected_size, expected_sha256).await? {
            return Ok(destination.to_path_buf());
        }
        let part = part_path(destination)?;
        let mut offset = match tokio::fs::metadata(&part).await {
            Ok(metadata) if metadata.len() <= expected_size => metadata.len(),
            Ok(_) => {
                tokio::fs::remove_file(&part).await.map_err(file_error)?;
                0
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
            Err(error) => return Err(file_error(error)),
        };
        ensure_disk_space(parent, expected_size.saturating_sub(offset))?;

        let mut request = self.client.get(url);
        if offset > 0 {
            request = request.header(RANGE, format!("bytes={offset}-"));
        }
        let response = request.send().await.map_err(download_error)?;
        let status = response.status();
        if !status.is_success() {
            tracing::warn!(status = status.as_u16(), "téléchargement du modèle refusé");
            return Err(AppError::Provider(
                "Le téléchargement de l'IA locale a échoué. Réessayez plus tard.".into(),
            ));
        }
        let resumed = offset > 0
            && status == reqwest::StatusCode::PARTIAL_CONTENT
            && valid_content_range(response.headers().get(CONTENT_RANGE), offset);
        if offset > 0 && !resumed {
            offset = 0;
        }
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(resumed)
            .truncate(!resumed)
            .open(&part)
            .await
            .map_err(file_error)?;
        let started = Instant::now();
        let mut downloaded = offset;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = tokio::select! {
            _ = cancellation.cancelled() => return Err(AppError::Cancelled),
            chunk = stream.next() => chunk,
        } {
            let chunk = chunk.map_err(download_error)?;
            file.write_all(&chunk).await.map_err(file_error)?;
            downloaded = downloaded.saturating_add(u64::try_from(chunk.len()).unwrap_or(u64::MAX));
            if downloaded > expected_size {
                drop(file);
                let _ = tokio::fs::remove_file(&part).await;
                return Err(AppError::Provider(
                    "Le fichier téléchargé ne correspond pas au modèle attendu.".into(),
                ));
            }
            let elapsed = started.elapsed().as_secs_f64().max(0.001);
            let transferred = downloaded.saturating_sub(offset);
            progress(LocalAiDownloadProgress {
                model_id,
                state: LocalAiState::Downloading,
                downloaded_bytes: downloaded,
                total_bytes: expected_size,
                bytes_per_second: (transferred as f64 / elapsed) as u64,
                progress: percent(downloaded, expected_size),
            });
        }
        file.flush().await.map_err(file_error)?;
        file.sync_all().await.map_err(file_error)?;
        drop(file);
        if downloaded != expected_size {
            return Err(AppError::Provider(format!(
                "Le téléchargement est incomplet ({downloaded} octets sur {expected_size}). Il pourra reprendre au prochain essai."
            )));
        }
        progress(LocalAiDownloadProgress {
            model_id,
            state: LocalAiState::Verifying,
            downloaded_bytes: downloaded,
            total_bytes: expected_size,
            bytes_per_second: 0,
            progress: 100,
        });
        let part_for_hash = part.clone();
        let digest = tokio::task::spawn_blocking(move || sha256_file(&part_for_hash))
            .await
            .map_err(|error| AppError::Provider(error.to_string()))??;
        if !digest.eq_ignore_ascii_case(expected_sha256) {
            let _ = tokio::fs::remove_file(&part).await;
            tracing::error!(model = ?model_id, "empreinte du modèle local invalide");
            return Err(AppError::Provider(
                "La vérification de l'IA locale a échoué. Le fichier invalide a été supprimé."
                    .into(),
            ));
        }
        tokio::fs::rename(&part, destination)
            .await
            .map_err(file_error)?;
        Ok(destination.to_path_buf())
    }
}

fn validate_source(url: &str) -> AppResult<()> {
    let parsed = reqwest::Url::parse(url)
        .map_err(|_| AppError::Validation("L'adresse du modèle local est invalide.".into()))?;
    if parsed.scheme() != "https" || parsed.host_str() != Some("huggingface.co") {
        return Err(AppError::Validation(
            "La source du modèle local n'est pas autorisée.".into(),
        ));
    }
    Ok(())
}

fn allowed_download_host(host: Option<&str>) -> bool {
    host.is_some_and(|host| {
        host == "huggingface.co"
            || host.ends_with(".huggingface.co")
            || host == "hf.co"
            || host.ends_with(".hf.co")
            || host.ends_with(".xethub.hf.co")
    })
}

async fn valid_existing_model(
    destination: &Path,
    expected_size: u64,
    expected_sha256: &str,
) -> AppResult<bool> {
    let metadata = match tokio::fs::metadata(destination).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(file_error(error)),
    };
    if metadata.is_file() && metadata.len() == expected_size {
        let path = destination.to_path_buf();
        let digest = tokio::task::spawn_blocking(move || sha256_file(&path))
            .await
            .map_err(|error| AppError::Provider(error.to_string()))??;
        if digest.eq_ignore_ascii_case(expected_sha256) {
            return Ok(true);
        }
    }
    tokio::fs::remove_file(destination)
        .await
        .map_err(file_error)?;
    Ok(false)
}

fn validate_destination(destination: &Path) -> AppResult<()> {
    let filename = destination
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if filename.is_empty()
        || !filename.ends_with(".gguf")
        || filename.contains('/')
        || filename.contains('\\')
        || filename == ".gguf"
    {
        return Err(AppError::Validation(
            "Le nom du modèle local est invalide.".into(),
        ));
    }
    Ok(())
}

fn part_path(destination: &Path) -> AppResult<PathBuf> {
    let filename = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| AppError::Validation("Le nom du modèle local est invalide.".into()))?;
    Ok(destination.with_file_name(format!("{filename}.part")))
}

fn ensure_disk_space(parent: &Path, remaining: u64) -> AppResult<()> {
    let available = fs2::available_space(parent).map_err(file_error)?;
    if !has_disk_space(available, remaining) {
        return Err(AppError::Validation(
            "L'espace disque est insuffisant pour installer l'IA locale.".into(),
        ));
    }
    Ok(())
}

fn has_disk_space(available: u64, remaining: u64) -> bool {
    available >= remaining.saturating_add(DISK_RESERVE_BYTES)
}

fn valid_content_range(value: Option<&reqwest::header::HeaderValue>, offset: u64) -> bool {
    value
        .and_then(|header| header.to_str().ok())
        .is_some_and(|value| value.starts_with(&format!("bytes {offset}-")))
}

pub(super) fn sha256_file(path: &Path) -> AppResult<String> {
    let file = std::fs::File::open(path).map_err(file_error)?;
    let mut reader = std::io::BufReader::with_capacity(1024 * 1024, file);
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = reader.read(&mut buffer).map_err(file_error)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn percent(downloaded: u64, total: u64) -> u8 {
    if total == 0 {
        return 0;
    }
    u8::try_from(downloaded.saturating_mul(100) / total)
        .unwrap_or(100)
        .min(100)
}

fn download_error(error: reqwest::Error) -> AppError {
    tracing::warn!(
        is_timeout = error.is_timeout(),
        is_connect = error.is_connect(),
        status = ?error.status().map(|status| status.as_u16()),
        "téléchargement du modèle interrompu"
    );
    AppError::Provider(
        "Le téléchargement de l'IA locale a été interrompu. Il reprendra au prochain essai.".into(),
    )
}

fn file_error(error: std::io::Error) -> AppError {
    tracing::error!(%error, "accès au fichier du modèle local impossible");
    AppError::Provider("Candilog ne peut pas écrire le modèle dans son dossier de données.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    fn digest(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    fn http_server(
        status: &str,
        headers: &[(&str, &str)],
        body: &'static [u8],
    ) -> (String, Arc<Mutex<String>>, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let received = Arc::new(Mutex::new(String::new()));
        let request_copy = Arc::clone(&received);
        let status = status.to_owned();
        let headers = headers
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect::<Vec<_>>();
        let handle = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut request = [0_u8; 4096];
            let size = socket.read(&mut request).unwrap();
            *request_copy.lock().unwrap() = String::from_utf8_lossy(&request[..size]).into_owned();
            let mut response = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\n", body.len());
            for (name, value) in headers {
                response.push_str(&format!("{name}: {value}\r\n"));
            }
            response.push_str("Connection: close\r\n\r\n");
            socket.write_all(response.as_bytes()).unwrap();
            socket.write_all(body).unwrap();
        });
        (format!("http://{address}/model.gguf"), received, handle)
    }

    fn test_downloader() -> ModelDownloader {
        ModelDownloader::with_client(reqwest::Client::new())
    }

    #[test]
    fn part_file_is_never_the_final_model() {
        let destination = Path::new("/tmp/ministral-8b-q4_k_m.gguf");
        assert_eq!(
            part_path(destination).unwrap(),
            Path::new("/tmp/ministral-8b-q4_k_m.gguf.part")
        );
    }

    #[test]
    fn path_traversal_filename_is_rejected() {
        assert!(validate_destination(Path::new("/tmp/model.bin")).is_err());
        assert!(validate_destination(Path::new("/tmp/.gguf")).is_err());
    }

    #[test]
    fn sha256_is_streamed_and_exact() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("small.gguf.part");
        std::fs::write(&path, b"Candilog").unwrap();
        assert_eq!(
            sha256_file(&path).unwrap(),
            "2ec7c8f59f28741f3aa2408e1a33545f3e1443412a1505c60fd744987998cbda"
        );
    }

    #[test]
    fn source_and_redirect_hosts_are_restricted() {
        assert!(
            validate_source("https://huggingface.co/mistralai/model/resolve/rev/model.gguf")
                .is_ok()
        );
        assert!(validate_source("http://huggingface.co/model.gguf").is_err());
        assert!(validate_source("https://example.com/model.gguf").is_err());
        assert!(allowed_download_host(Some("cdn-lfs-us-1.hf.co")));
        assert!(allowed_download_host(Some("cas-bridge.xethub.hf.co")));
        assert!(!allowed_download_host(Some("hf.co.example.com")));
    }

    #[test]
    fn insufficient_disk_space_is_detected_with_reserve() {
        assert!(!has_disk_space(DISK_RESERVE_BYTES + 99, 100));
        assert!(has_disk_space(DISK_RESERVE_BYTES + 100, 100));
    }

    #[tokio::test]
    async fn successful_download_reports_progress_and_atomically_renames() {
        let bytes = b"Candilog";
        let (url, _, server) = http_server("200 OK", &[], bytes);
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("model.gguf");
        let events = Arc::new(Mutex::new(Vec::new()));
        let event_copy = Arc::clone(&events);
        let result = test_downloader()
            .download(
                ModelDownload {
                    model_id: LocalModelId::Ministral3Light,
                    url: &url,
                    destination: &destination,
                    expected_size: bytes.len() as u64,
                    expected_sha256: &digest(bytes),
                },
                &CancellationToken::new(),
                move |event| event_copy.lock().unwrap().push(event),
            )
            .await
            .unwrap();
        server.join().unwrap();
        assert_eq!(result, destination);
        assert_eq!(std::fs::read(&destination).unwrap(), bytes);
        assert!(!part_path(&destination).unwrap().exists());
        let events = events.lock().unwrap();
        assert!(events.iter().any(|event| event.progress == 100));
        assert_eq!(events.last().unwrap().state, LocalAiState::Verifying);
    }

    #[tokio::test]
    async fn range_resume_appends_to_part_file() {
        let all = b"Candilog";
        let (url, request, server) = http_server(
            "206 Partial Content",
            &[("Content-Range", "bytes 4-7/8")],
            b"ilog",
        );
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("model.gguf");
        std::fs::write(part_path(&destination).unwrap(), b"Cand").unwrap();
        test_downloader()
            .download(
                ModelDownload {
                    model_id: LocalModelId::Ministral3Light,
                    url: &url,
                    destination: &destination,
                    expected_size: all.len() as u64,
                    expected_sha256: &digest(all),
                },
                &CancellationToken::new(),
                |_| {},
            )
            .await
            .unwrap();
        server.join().unwrap();
        assert!(request.lock().unwrap().contains("range: bytes=4-"));
        assert_eq!(std::fs::read(destination).unwrap(), all);
    }

    #[tokio::test]
    async fn invalid_checksum_removes_part_and_never_publishes_model() {
        let bytes = b"invalid";
        let (url, _, server) = http_server("200 OK", &[], bytes);
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("model.gguf");
        let result = test_downloader()
            .download(
                ModelDownload {
                    model_id: LocalModelId::Ministral3Light,
                    url: &url,
                    destination: &destination,
                    expected_size: bytes.len() as u64,
                    expected_sha256: &"0".repeat(64),
                },
                &CancellationToken::new(),
                |_| {},
            )
            .await;
        server.join().unwrap();
        assert!(result.is_err());
        assert!(!destination.exists());
        assert!(!part_path(&destination).unwrap().exists());
    }

    #[tokio::test]
    async fn http_error_does_not_publish_a_model() {
        let (url, _, server) = http_server("503 Service Unavailable", &[], b"");
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("model.gguf");
        let result = test_downloader()
            .download(
                ModelDownload {
                    model_id: LocalModelId::Ministral3Light,
                    url: &url,
                    destination: &destination,
                    expected_size: 8,
                    expected_sha256: &"0".repeat(64),
                },
                &CancellationToken::new(),
                |_| {},
            )
            .await;
        server.join().unwrap();
        assert!(result.is_err());
        assert!(!destination.exists());
    }

    #[tokio::test]
    async fn cancellation_keeps_part_for_a_future_resume() {
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("model.gguf");
        let part = part_path(&destination).unwrap();
        std::fs::write(&part, b"Cand").unwrap();
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let result = test_downloader()
            .download(
                ModelDownload {
                    model_id: LocalModelId::Ministral3Light,
                    url: "http://127.0.0.1:1/model.gguf",
                    destination: &destination,
                    expected_size: 8,
                    expected_sha256: &digest(b"Candilog"),
                },
                &cancellation,
                |_| {},
            )
            .await;
        assert!(matches!(result, Err(AppError::Cancelled)));
        assert_eq!(std::fs::read(part).unwrap(), b"Cand");
        assert!(!destination.exists());
    }

    #[tokio::test]
    async fn an_already_verified_model_is_not_downloaded_again() {
        let bytes = b"Candilog";
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("model.gguf");
        std::fs::write(&destination, bytes).unwrap();
        let result = test_downloader()
            .download(
                ModelDownload {
                    model_id: LocalModelId::Ministral3Light,
                    url: "http://127.0.0.1:1/model.gguf",
                    destination: &destination,
                    expected_size: bytes.len() as u64,
                    expected_sha256: &digest(bytes),
                },
                &CancellationToken::new(),
                |_| {},
            )
            .await
            .unwrap();
        assert_eq!(result, destination);
    }
}
