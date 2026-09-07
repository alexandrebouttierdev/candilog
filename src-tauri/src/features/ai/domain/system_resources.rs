//! Snapshot des ressources machine pour le rail (CPU / RAM / VRAM).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Charge CPU, RAM et VRAM (ou mémoire unifiée) à un instant donné.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "ai.ts")]
pub struct SystemResourceSnapshot {
    pub cpu_percent: f32,
    pub ram_used_percent: f32,
    #[ts(type = "number")]
    pub ram_used_mb: u64,
    #[ts(type = "number")]
    pub ram_total_mb: u64,
    pub vram_available: bool,
    pub vram_used_percent: Option<f32>,
    #[ts(type = "number | null")]
    pub vram_used_mb: Option<u64>,
    #[ts(type = "number | null")]
    pub vram_total_mb: Option<u64>,
}
