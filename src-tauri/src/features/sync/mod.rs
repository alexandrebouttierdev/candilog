//! Synchronisation facultative entre ordinateurs (`docs/SYNC.md`).
//!
//! Le mode local reste celui par défaut. Rattaché à un espace — dossier personnel, Google
//! Drive ou OneDrive —, l'ordinateur y publie des lots chiffrés de ses modifications et
//! applique ceux des autres. SQLite n'est jamais copié comme un fichier actif.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
