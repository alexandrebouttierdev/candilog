//! Bibliothèque de composants desktop de Candilog.
//!
//! Chaque module a une responsabilité unique, ne connaît ni l'état global ni
//! SQLite, et ne porte aucune règle métier : il reçoit des valeurs déjà
//! préparées et rend une composition Iced.

pub mod ambient;
pub mod avatar;
pub mod badge;
pub mod badge_score;
pub mod bar;
pub mod button;
pub mod choice;
pub mod document;
pub mod donut;
pub mod field;
pub mod header;
pub mod icon;
pub mod inspector;
pub mod layout;
pub mod list;
pub mod meter;
pub mod notification;
pub mod overlay;
pub mod pagination;
pub mod provider_icon;
pub mod runtime_status;
pub mod score_gauge;
pub mod sidebar;
pub mod skeleton;
pub mod sparkline;
pub mod stat_card;
pub mod state;
pub mod surface;
pub mod table;
pub mod tabs;
pub mod titlebar;
pub mod tooltip;
pub mod typo;
pub mod workflow;
