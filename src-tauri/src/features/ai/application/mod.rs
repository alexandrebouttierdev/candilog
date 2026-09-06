//! Orchestration des workflows IA.

mod local_selector;
mod local_service;
mod service;
pub use local_selector::LocalModelSelector;
pub use local_service::LocalAiService;
pub use service::AiService;
