//! Domaine de la synchronisation : protocole, versions, arbitrage, contrat des fournisseurs.
//! Ni Tauri, ni rusqlite, ni client HTTP.

pub mod entities;
pub mod merge;
pub mod model;
pub mod protocol;
pub mod remote;
pub mod version;

pub use merge::{ConflictKind, Decision, FieldConflict, LocalView, RecordMeta};
pub use model::{
    ConflictChoice, ConflictField, ProviderAvailability, RemoteProbe, SetupChoice, SetupOutcome,
    SetupRequest, SyncConflict, SyncPhase, SyncProviderKind, SyncStatus,
};
pub use remote::{PutOutcome, RemoteError, RemoteObject, RemoteStore};
pub use version::{Causality, VersionVector};
