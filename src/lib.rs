//! Bibliothèque de Candilog Desktop, application native Rust + Iced.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod app;
pub mod core;
pub mod modules;
pub mod navigation;
pub mod shared;
pub mod ui;
