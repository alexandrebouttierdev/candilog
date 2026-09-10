//! Manifeste des binaires Ollama officiels supportés par Candilog.

use crate::core::errors::{AppError, AppResult};
use crate::features::ai::domain::MANAGED_OLLAMA_RUNTIME_VERSION;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeArtifact {
    pub version: &'static str,
    pub url: &'static str,
    pub sha256: &'static str,
    pub archive_name: &'static str,
    pub executable: &'static str,
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
            executable: "bin/ollama",
        });
    }
    if cfg!(target_os = "macos") {
        return Some(RuntimeArtifact {
            version: MANAGED_OLLAMA_RUNTIME_VERSION,
            url: "https://github.com/ollama/ollama/releases/download/v0.13.4/ollama-darwin.tgz",
            sha256: "4831b6b3b0b736b9abf06adbaeaffc44c7e363fcfdd120f9573be00b5691f0e9",
            archive_name: "ollama-darwin.tgz",
            executable: "bin/ollama",
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
    fn plateforme_inconnue_retourne_none() {
        if cfg!(target_os = "windows") {
            assert!(runtime_artifact_for_current_platform().is_none());
        }
    }

    #[test]
    fn require_runtime_artifact_message_clair_sans_support() {
        if cfg!(target_os = "windows") {
            let error = require_runtime_artifact().unwrap_err();
            assert!(error.to_string().contains("pas encore disponible"));
        }
    }
}
