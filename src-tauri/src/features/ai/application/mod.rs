//! Orchestration des workflows IA.

mod managed_ollama_service;
mod service;
pub use managed_ollama_service::{ManagedOllamaPaths, ManagedOllamaService};
pub use service::AiService;
