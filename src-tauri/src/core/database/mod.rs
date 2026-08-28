//! Pool `SQLite`, migrations locales et helpers partagés par les dépôts.

pub mod connection;
pub mod helpers;

pub use connection::{open_pool, run_local_migrations, SqlitePool, DERNIERE_VERSION};
