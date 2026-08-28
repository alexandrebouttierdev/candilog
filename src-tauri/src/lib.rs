//! Candilog — backend local de suivi de candidatures.
//!
//! Organisation : `core` porte le socle technique, `features` le métier découpé par
//! fonctionnalité (domain / application / infrastructure / presentation), `infrastructure`
//! les implémentations transverses, `app` la composition et le démarrage Tauri.

// Convention reprise de l'application Iced : une panique dans le métier ferme la fenêtre
// sans message exploitable. Les erreurs remontent par `AppResult`, jamais par `unwrap`.
// Les tests y échappent : un `unwrap` y est une assertion, et son échec est le résultat.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod app;
pub mod core;
pub mod features;
pub mod infrastructure;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    app::bootstrap::run();
}
