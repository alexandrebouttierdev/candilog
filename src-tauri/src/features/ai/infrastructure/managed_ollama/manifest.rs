//! Manifeste des binaires Ollama officiels supportés par Candilog.

use crate::core::errors::{AppError, AppResult};
use crate::features::ai::domain::MANAGED_OLLAMA_RUNTIME_VERSION;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeArtifact {
    pub version: &'static str,
    pub url: &'static str,
    pub sha256: &'static str,
    pub archive_name: &'static str,
    pub archive_format: RuntimeArchiveFormat,
    pub executable: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeArchiveFormat {
    Tgz,
    Zip,
}

#[must_use]
pub fn runtime_artifact_for_current_platform() -> Option<RuntimeArtifact> {
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Some(RuntimeArtifact {
            version: MANAGED_OLLAMA_RUNTIME_VERSION,
            url:
                "https://github.com/ollama/ollama/releases/download/v0.13.4/ollama-linux-amd64.tgz",
            sha256: "c9c78d2cff13dee8397f59e53cbc75af2b64e55fb5b59f0ae25dc6a36bc0ebab",
            archive_name: "ollama-linux-amd64.tgz",
            archive_format: RuntimeArchiveFormat::Tgz,
            executable: "bin/ollama",
        });
    }
    if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        return Some(RuntimeArtifact {
            version: MANAGED_OLLAMA_RUNTIME_VERSION,
            url:
                "https://github.com/ollama/ollama/releases/download/v0.13.4/ollama-linux-arm64.tgz",
            sha256: "a129cd601978a94b13133e5c788a65e7930c265a3c23cb1701a7190877000888",
            archive_name: "ollama-linux-arm64.tgz",
            archive_format: RuntimeArchiveFormat::Tgz,
            executable: "bin/ollama",
        });
    }
    if cfg!(target_os = "macos") {
        return Some(RuntimeArtifact {
            version: MANAGED_OLLAMA_RUNTIME_VERSION,
            url: "https://github.com/ollama/ollama/releases/download/v0.13.4/ollama-darwin.tgz",
            sha256: "69251adfdc02b82bbafa827a40af5f86ce671a30a9eb86470b4ae9b6184dc66b",
            archive_name: "ollama-darwin.tgz",
            archive_format: RuntimeArchiveFormat::Tgz,
            executable: "bin/ollama",
        });
    }
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        return Some(RuntimeArtifact {
            version: MANAGED_OLLAMA_RUNTIME_VERSION,
            url:
                "https://github.com/ollama/ollama/releases/download/v0.13.4/ollama-windows-amd64.zip",
            sha256: "bde5623b27d897a205eab0795172bcec37894d3822052b0ebe2011500072ce55",
            archive_name: "ollama-windows-amd64.zip",
            archive_format: RuntimeArchiveFormat::Zip,
            executable: "ollama.exe",
        });
    }
    if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
        return Some(RuntimeArtifact {
            version: MANAGED_OLLAMA_RUNTIME_VERSION,
            url:
                "https://github.com/ollama/ollama/releases/download/v0.13.4/ollama-windows-arm64.zip",
            sha256: "52e43ec0eb957b493cbdda1c390357b1ca28a573a91fdd339a6d92997f575c69",
            archive_name: "ollama-windows-arm64.zip",
            archive_format: RuntimeArchiveFormat::Zip,
            executable: "ollama.exe",
        });
    }
    None
}

pub fn require_runtime_artifact() -> AppResult<RuntimeArtifact> {
    runtime_artifact_for_current_platform().ok_or_else(|| {
        AppError::Provider(
            "L'IA locale Candilog n'est pas encore disponible sur cette plateforme.".into(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_amd64_a_un_artifact_verifie() {
        if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            let artifact = runtime_artifact_for_current_platform().expect("artifact linux amd64");
            assert_eq!(artifact.version, MANAGED_OLLAMA_RUNTIME_VERSION);
            assert!(artifact.url.contains("ollama-linux-amd64.tgz"));
            assert_eq!(
                artifact.sha256,
                "c9c78d2cff13dee8397f59e53cbc75af2b64e55fb5b59f0ae25dc6a36bc0ebab"
            );
        }
    }

    #[test]
    fn windows_x64_a_un_artifact_zip_verifie() {
        if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            let artifact = runtime_artifact_for_current_platform().expect("artifact Windows x64");
            assert_eq!(artifact.version, MANAGED_OLLAMA_RUNTIME_VERSION);
            assert_eq!(artifact.archive_format, RuntimeArchiveFormat::Zip);
            assert_eq!(artifact.executable, "ollama.exe");
            assert_eq!(
                artifact.sha256,
                "bde5623b27d897a205eab0795172bcec37894d3822052b0ebe2011500072ce55"
            );
        }
    }

    #[test]
    fn windows_arm64_a_un_artifact_zip_verifie() {
        if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
            let artifact = runtime_artifact_for_current_platform().expect("artifact Windows ARM64");
            assert_eq!(artifact.version, MANAGED_OLLAMA_RUNTIME_VERSION);
            assert_eq!(artifact.archive_format, RuntimeArchiveFormat::Zip);
            assert_eq!(artifact.executable, "ollama.exe");
            assert_eq!(
                artifact.sha256,
                "52e43ec0eb957b493cbdda1c390357b1ca28a573a91fdd339a6d92997f575c69"
            );
        }
    }
}
