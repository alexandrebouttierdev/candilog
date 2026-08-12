//! État, messages, mise à jour et vues Iced de Candilog.
//!
//! `update.rs` agrégeait six responsabilités sans rapport — harnais de capture visuelle,
//! coquille de fenêtre, formulaires métier, orchestration IA, chaîne de mise à jour et
//! utilitaires divers — dans un même fichier, ce qui maximisait les conflits et rendait la
//! relecture difficile. Chacune vit désormais dans son module.

mod capture;
mod commandes;
mod coquille;
mod export;
pub mod message;
mod profile_edit;
mod snapshot;
pub mod state;
mod update;
mod view;

pub use coquille::{subscription, theme};
pub use message::Message;
pub use state::App;
pub use update::update;
pub use view::view;
