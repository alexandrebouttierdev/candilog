//! Implémentations concrètes des fournisseurs LLM.

pub mod claude;
pub mod gemini;
pub mod ollama;
pub mod openai_compat;

pub use claude::ClaudeProvider;
pub use gemini::GeminiProvider;
pub use ollama::OllamaProvider;
pub use openai_compat::OpenAiCompatProvider;
