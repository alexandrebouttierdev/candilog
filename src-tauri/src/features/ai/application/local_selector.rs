//! Sélection conservatrice du modèle et du backend à partir du matériel détecté.

use crate::features::ai::domain::{
    LocalAiBackend, LocalAiHardware, LocalAiRecommendation, LocalModelDefinition,
    LocalModelEvaluation, LocalModelId, LocalModelProfile, ModelCompatibility, ModelRegistry,
};

const CPU_MEMORY_PERCENT: u64 = 68;
const GPU_MEMORY_PERCENT: u64 = 80;

pub struct LocalModelSelector;

impl LocalModelSelector {
    #[must_use]
    pub fn recommend(hardware: LocalAiHardware) -> LocalAiRecommendation {
        let backend = preferred_backend(&hardware);
        let mut models = ModelRegistry::all();
        models.reverse();

        let mut selected = None;
        let mut evaluations = Vec::with_capacity(models.len());
        for model in models {
            let (comfortable, can_load, reason) = compatibility_for(&hardware, backend, &model);
            let compatibility = if comfortable && selected.is_none() {
                selected = Some(model.clone());
                ModelCompatibility::Optimal
            } else if comfortable {
                ModelCompatibility::Supported
            } else if can_load {
                ModelCompatibility::NotRecommended
            } else {
                ModelCompatibility::Unsupported
            };
            evaluations.push(LocalModelEvaluation {
                model,
                compatibility,
                reason,
            });
        }

        let reason = selected.as_ref().map_or_else(
            || "Aucun profil ne dispose d'une marge mémoire et CPU suffisante pour garantir une utilisation fluide.".into(),
            |model| format!("Le profil {} conserve une marge confortable pour Candilog et le système.", profile_label(model.profile)),
        );
        LocalAiRecommendation {
            hardware,
            selected_model: selected,
            backend,
            evaluations,
            reason,
        }
    }
}

fn preferred_backend(hardware: &LocalAiHardware) -> LocalAiBackend {
    if hardware.apple_silicon && hardware.metal {
        return LocalAiBackend::Metal;
    }
    let best = hardware
        .gpus
        .iter()
        .filter_map(|gpu| gpu.backend.zip(gpu.total_vram_mb))
        .max_by_key(|(_, memory)| *memory)
        .map(|(backend, _)| backend);
    best.unwrap_or(LocalAiBackend::Cpu)
}

fn compatibility_for(
    hardware: &LocalAiHardware,
    backend: LocalAiBackend,
    model: &LocalModelDefinition,
) -> (bool, bool, String) {
    if hardware.apple_silicon {
        return unified_memory_compatibility(hardware, model);
    }
    if backend != LocalAiBackend::Cpu {
        return gpu_compatibility(hardware, backend, model);
    }
    cpu_compatibility(hardware, model)
}

fn unified_memory_compatibility(
    hardware: &LocalAiHardware,
    model: &LocalModelDefinition,
) -> (bool, bool, String) {
    let total = hardware.unified_memory_mb.unwrap_or(hardware.total_ram_mb);
    let budget = total.saturating_mul(CPU_MEMORY_PERCENT) / 100;
    let available = hardware.available_ram_mb;
    let comfortable = total >= model.recommended_ram_mb
        && budget >= model.estimated_ram_mb
        && available >= model.estimated_ram_mb;
    let can_load = budget >= model.estimated_ram_mb;
    (
        comfortable,
        can_load,
        format!(
            "{} Mo de mémoire unifiée, {} Mo disponibles ; {} Mo recommandés.",
            total, available, model.recommended_ram_mb
        ),
    )
}

fn gpu_compatibility(
    hardware: &LocalAiHardware,
    backend: LocalAiBackend,
    model: &LocalModelDefinition,
) -> (bool, bool, String) {
    let gpu = hardware
        .gpus
        .iter()
        .filter(|gpu| gpu.backend == Some(backend))
        .max_by_key(|gpu| gpu.total_vram_mb.unwrap_or_default());
    let total = gpu.and_then(|gpu| gpu.total_vram_mb).unwrap_or_default();
    let available = gpu.and_then(|gpu| gpu.available_vram_mb).unwrap_or(total);
    let budget = total.saturating_mul(GPU_MEMORY_PERCENT) / 100;
    let load_estimate = model.download_size_bytes / 1_048_576 + 768;
    let recommended = model
        .recommended_vram_mb
        .unwrap_or(model.recommended_ram_mb);
    let comfortable = total >= recommended && budget >= load_estimate && available >= load_estimate;
    let can_load = budget >= load_estimate;
    (
        comfortable,
        can_load,
        format!(
            "{} Mo de mémoire graphique, {} Mo disponibles ; {} Mo recommandés.",
            total, available, recommended
        ),
    )
}

fn cpu_compatibility(
    hardware: &LocalAiHardware,
    model: &LocalModelDefinition,
) -> (bool, bool, String) {
    let total = hardware.total_ram_mb;
    let available = hardware.available_ram_mb;
    let budget = total.saturating_mul(CPU_MEMORY_PERCENT) / 100;
    let physical = hardware.physical_cores.unwrap_or(hardware.logical_cores);
    let minimum_cores = match model.id {
        LocalModelId::Ministral3Light => 4,
        LocalModelId::Ministral3Balanced => 6,
        LocalModelId::Ministral3Quality => 8,
    };
    let comfortable = total >= model.recommended_ram_mb
        && budget >= model.estimated_ram_mb
        && available >= model.estimated_ram_mb
        && physical >= minimum_cores;
    let can_load = budget >= model.estimated_ram_mb;
    (
        comfortable,
        can_load,
        format!(
            "{} Mo de RAM, {} Mo disponibles et {} cœurs physiques ; {} Mo et {} cœurs recommandés.",
            total, available, physical, model.recommended_ram_mb, minimum_cores
        ),
    )
}

const fn profile_label(profile: LocalModelProfile) -> &'static str {
    match profile {
        LocalModelProfile::Light => "Léger",
        LocalModelProfile::Balanced => "Équilibré",
        LocalModelProfile::Quality => "Qualité",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::ai::domain::LocalGpuInfo;

    fn cpu(total_ram_mb: u64) -> LocalAiHardware {
        LocalAiHardware {
            os: "linux".into(),
            architecture: "x86_64".into(),
            cpu: "x86_64".into(),
            cpu_model: "CPU de test".into(),
            logical_cores: 16,
            physical_cores: Some(8),
            total_ram_mb,
            available_ram_mb: total_ram_mb,
            gpus: vec![],
            apple_silicon: false,
            soc_model: None,
            unified_memory_mb: None,
            metal: false,
            cuda: false,
            vulkan: false,
        }
    }

    fn apple(memory_mb: u64) -> LocalAiHardware {
        LocalAiHardware {
            apple_silicon: true,
            unified_memory_mb: Some(memory_mb),
            metal: true,
            os: "macos".into(),
            architecture: "aarch64".into(),
            ..cpu(memory_mb)
        }
    }

    fn gpu(vram_mb: u64) -> LocalAiHardware {
        LocalAiHardware {
            cuda: true,
            gpus: vec![LocalGpuInfo {
                name: "GPU de test".into(),
                vendor: "NVIDIA".into(),
                total_vram_mb: Some(vram_mb),
                available_vram_mb: Some(vram_mb),
                backend: Some(LocalAiBackend::Cuda),
            }],
            ..cpu(32_768)
        }
    }

    fn selected(hardware: LocalAiHardware) -> Option<LocalModelProfile> {
        LocalModelSelector::recommend(hardware)
            .selected_model
            .map(|model| model.profile)
    }

    #[test]
    fn cpu_thresholds() {
        assert_eq!(selected(cpu(8_192)), Some(LocalModelProfile::Light));
        assert_eq!(selected(cpu(16_384)), Some(LocalModelProfile::Balanced));
        assert_eq!(selected(cpu(24_576)), Some(LocalModelProfile::Quality));
        assert_eq!(selected(cpu(8_191)), None);
        assert_eq!(selected(cpu(16_383)), Some(LocalModelProfile::Light));
        assert_eq!(selected(cpu(24_575)), Some(LocalModelProfile::Balanced));
        assert_eq!(selected(cpu(24_577)), Some(LocalModelProfile::Quality));
    }

    #[test]
    fn apple_silicon_thresholds() {
        assert_eq!(selected(apple(8_192)), Some(LocalModelProfile::Light));
        assert_eq!(selected(apple(16_384)), Some(LocalModelProfile::Balanced));
        assert_eq!(selected(apple(24_576)), Some(LocalModelProfile::Quality));
        assert_eq!(selected(apple(8_191)), None);
        assert_eq!(selected(apple(16_383)), Some(LocalModelProfile::Light));
        assert_eq!(selected(apple(24_575)), Some(LocalModelProfile::Balanced));
    }

    #[test]
    fn dedicated_gpu_thresholds() {
        assert_eq!(selected(gpu(4_096)), Some(LocalModelProfile::Light));
        assert_eq!(selected(gpu(8_192)), Some(LocalModelProfile::Balanced));
        assert_eq!(selected(gpu(12_288)), Some(LocalModelProfile::Quality));
        assert_eq!(selected(gpu(4_095)), None);
        assert_eq!(selected(gpu(8_191)), Some(LocalModelProfile::Light));
        assert_eq!(selected(gpu(12_287)), Some(LocalModelProfile::Balanced));
        assert_eq!(selected(gpu(12_289)), Some(LocalModelProfile::Quality));
    }

    #[test]
    fn low_end_machine_has_no_automatic_model() {
        assert_eq!(selected(cpu(6_000)), None);
    }

    #[test]
    fn slow_cpu_does_not_select_a_heavy_model_only_because_it_fits() {
        let mut hardware = cpu(24_576);
        hardware.physical_cores = Some(4);
        assert_eq!(selected(hardware), Some(LocalModelProfile::Light));
    }
}
