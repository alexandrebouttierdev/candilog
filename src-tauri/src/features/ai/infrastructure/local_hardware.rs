//! Détection matérielle sans commande ni runtime installé par l'utilisateur.

use crate::features::ai::domain::{LocalAiBackend, LocalAiHardware, LocalGpuInfo};
use llama_cpp_2::{list_llama_ggml_backend_devices, LlamaBackendDevice, LlamaBackendDeviceType};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

#[must_use]
pub fn detect_local_ai_hardware() -> LocalAiHardware {
    let mut system = System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::everything())
            .with_cpu(CpuRefreshKind::everything()),
    );
    system.refresh_memory();
    system.refresh_cpu_all();

    let cpu = system.cpus().first();
    let architecture = System::cpu_arch();
    let apple_silicon =
        cfg!(target_os = "macos") && matches!(architecture.as_str(), "aarch64" | "arm64");
    // Le runtime n'est initialisé que lorsque l'utilisateur ouvre Mistral Local. Le
    // démarrage ordinaire de Candilog ne paie donc pas le coût du backend graphique.
    let _ = super::local_runtime::shared_backend();
    let devices = list_llama_ggml_backend_devices();
    let mut gpus = devices
        .iter()
        .filter(|device| {
            matches!(
                device.device_type,
                LlamaBackendDeviceType::Gpu
                    | LlamaBackendDeviceType::IntegratedGpu
                    | LlamaBackendDeviceType::Accelerator
            )
        })
        .map(|device| gpu_info(device, apple_silicon))
        .collect::<Vec<_>>();
    gpus.sort_by_key(|gpu| std::cmp::Reverse(gpu.total_vram_mb.unwrap_or_default()));
    let metal = devices
        .iter()
        .any(|device| backend(device) == Some(LocalAiBackend::Metal));
    let cuda = devices
        .iter()
        .any(|device| backend(device) == Some(LocalAiBackend::Cuda));
    let vulkan = devices
        .iter()
        .any(|device| backend(device) == Some(LocalAiBackend::Vulkan));
    let total_ram_mb = system.total_memory() / 1_048_576;

    LocalAiHardware {
        os: std::env::consts::OS.into(),
        architecture: architecture.clone(),
        cpu: cpu.map_or_else(
            || architecture.clone(),
            |value| value.vendor_id().to_owned(),
        ),
        cpu_model: cpu.map_or_else(
            || "Processeur inconnu".into(),
            |value| value.brand().to_owned(),
        ),
        logical_cores: u32::try_from(system.cpus().len()).unwrap_or(u32::MAX),
        physical_cores: System::physical_core_count()
            .map(|value| u32::try_from(value).unwrap_or(u32::MAX)),
        total_ram_mb,
        available_ram_mb: system.available_memory() / 1_048_576,
        gpus,
        apple_silicon,
        soc_model: apple_silicon.then(|| {
            cpu.map_or("Apple Silicon", |value| value.brand())
                .to_owned()
        }),
        unified_memory_mb: apple_silicon.then_some(total_ram_mb),
        metal,
        cuda,
        vulkan,
    }
}

fn gpu_info(device: &LlamaBackendDevice, apple_silicon: bool) -> LocalGpuInfo {
    let name = if device.description.trim().is_empty() {
        device.name.clone()
    } else {
        device.description.clone()
    };
    LocalGpuInfo {
        vendor: gpu_vendor(&name),
        name,
        // Sur Apple Silicon cette valeur décrit la mémoire unifiée, jamais une VRAM à
        // additionner à la RAM. Elle reste donc absente du contrat GPU.
        total_vram_mb: (!apple_silicon).then_some(device.memory_total as u64 / 1_048_576),
        available_vram_mb: (!apple_silicon).then_some(device.memory_free as u64 / 1_048_576),
        backend: backend(device),
    }
}

fn backend(device: &LlamaBackendDevice) -> Option<LocalAiBackend> {
    match device.backend.to_ascii_lowercase().as_str() {
        value if value.contains("metal") => Some(LocalAiBackend::Metal),
        value if value.contains("cuda") => Some(LocalAiBackend::Cuda),
        value if value.contains("vulkan") => Some(LocalAiBackend::Vulkan),
        value if value.contains("cpu") => Some(LocalAiBackend::Cpu),
        _ => None,
    }
}

fn gpu_vendor(name: &str) -> String {
    let normalized = name.to_ascii_lowercase();
    if normalized.contains("nvidia")
        || normalized.contains("geforce")
        || normalized.contains("quadro")
    {
        "NVIDIA"
    } else if normalized.contains("amd") || normalized.contains("radeon") {
        "AMD"
    } else if normalized.contains("intel") || normalized.contains("arc") {
        "Intel"
    } else if normalized.contains("apple") {
        "Apple"
    } else {
        "Inconnu"
    }
    .into()
}

/// RAM réellement disponible à l'instant de l'appel, en mégaoctets.
///
/// Volontairement distincte de [`detect_local_ai_hardware`] : le garde-fou mémoire est
/// évalué à chaque inférence et ne doit ni initialiser le backend llama.cpp ni énumérer
/// les périphériques.
#[must_use]
pub fn available_ram_mb() -> u64 {
    let mut system = System::new_with_specifics(
        RefreshKind::nothing().with_memory(MemoryRefreshKind::nothing().with_ram()),
    );
    system.refresh_memory();
    system.available_memory() / 1_048_576
}

/// Empreinte mémoire résidente du processus Candilog, en mégaoctets.
///
/// Ne rafraîchit que le processus courant : la mesure est prise à chaque inférence et ne
/// doit pas énumérer toute la table des processus.
#[must_use]
pub fn process_rss_mb() -> Option<u64> {
    let pid = sysinfo::get_current_pid().ok()?;
    let mut system = System::new();
    system.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&[pid]),
        false,
        sysinfo::ProcessRefreshKind::nothing().with_memory(),
    );
    system
        .process(pid)
        .map(|process| process.memory() / 1_048_576)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Une lecture mémoire muette renverrait zéro et ferait refuser *toute* inférence
    /// locale par le garde-fou. Ce test vérifie que le rafraîchissement ciblé alimente
    /// bien la valeur, et pas seulement qu'il compile.
    #[test]
    fn la_ram_disponible_est_reellement_mesuree() {
        let disponible = available_ram_mb();
        assert!(
            disponible > 0,
            "la RAM disponible mesurée est nulle : le garde-fou refuserait tout"
        );
    }

    /// Même piège pour l'empreinte du processus, qui alimente les lignes de journal
    /// encadrant chaque inférence.
    #[test]
    fn l_empreinte_du_processus_est_reellement_mesuree() {
        let rss = process_rss_mb().expect("le processus courant doit être lisible");
        assert!(rss > 0, "empreinte résidente nulle : la mesure est muette");
    }
}
