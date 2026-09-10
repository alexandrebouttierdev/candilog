//! Orchestration des workflows IA.

mod local_selector;
mod local_service;
mod managed_ollama_service;
mod service;
pub use local_selector::LocalModelSelector;
pub use local_service::LocalAiService;
pub use managed_ollama_service::{ManagedOllamaPaths, ManagedOllamaService};
pub use service::AiService;
