//! Runtime Ollama privé géré par Candilog.

mod api;
mod installer;
mod manifest;
mod process;

pub use api::ManagedOllamaApi;
pub use installer::RuntimeInstaller;
pub use manifest::require_runtime_artifact;
pub use process::ManagedOllamaProcess;
