//! Commande IPC du snapshot ressources système (rail).

use crate::features::ai::domain::SystemResourceSnapshot;
use crate::features::ai::infrastructure::take_system_resource_snapshot;

#[tauri::command(rename_all = "snake_case")]
pub fn system_resource_snapshot() -> SystemResourceSnapshot {
    take_system_resource_snapshot()
}
