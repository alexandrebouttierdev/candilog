//! Implémentations : état SQLite, chiffrement, fournisseurs (dossier, Google Drive,
//! OneDrive) et coffre des secrets.

pub mod apply;
pub mod authorized;
pub mod crypto;
pub mod folder;
pub mod google_drive;
#[cfg(test)]
pub mod memory;
pub mod oauth;
pub mod onedrive;
pub mod rows;
pub mod secrets;
pub mod store;

pub use store::{SqliteSyncStore, SyncConfig};
