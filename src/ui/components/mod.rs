//! Bibliothèque de composants desktop de Candilog.
//!
//! Chaque module a une responsabilité unique, ne connaît ni l'état global ni
//! SQLite, et ne porte aucune règle métier : il reçoit des valeurs déjà
//! préparées et rend une composition Iced.

pub mod badge;
pub mod button;
pub mod choice;
pub mod document;
pub mod field;
pub mod icon;
pub mod inspector;
pub mod layout;
pub mod list;
pub mod meter;
pub mod notification;
pub mod overlay;
pub mod pane;
pub mod rail;
pub mod sidebar;
pub mod state;
pub mod surface;
pub mod table;
pub mod toolbar;
pub mod tooltip;
pub mod typo;
